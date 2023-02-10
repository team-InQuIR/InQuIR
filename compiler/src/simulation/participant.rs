use inquir::{
    ParticipantId,
    Qubit, QubitKind,
    Value,
    Process,
};
use crate::simulation::{
    shared_memory::SharedMemory,
    comm_buffer::SendData,
    evaluation_cost::{EvaluationCost, collect_cost},
    latency::Latency,
};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;

#[derive(Debug, Clone)]
struct Registers {
    q: BinaryHeap<Reverse<(EvaluationCost, Qubit)>>,
    cq: HashMap<ParticipantId, BinaryHeap<Reverse<(EvaluationCost, Qubit)>>>,
    q_to_partner: HashMap<u32, ParticipantId>,
    used_q: HashSet<u32>,
    used_cq: HashSet<u32>,
}

impl Registers {
    pub fn new(num_q: usize, num_cq: HashMap<ParticipantId, u32>) -> Self {
        let q = (0..num_q).fold(BinaryHeap::new(), |mut h, i| {
            let q = Qubit::new(QubitKind::Data, i as u32);
            let initial_cost = Default::default();
            h.push(Reverse((initial_cost, q)));
            h
        });
        let mut cq = HashMap::new();
        let mut counter = 0;
        let mut q_to_partner = HashMap::new();
        num_cq.iter().for_each(|(partner, num)| {
            let mut h = BinaryHeap::new();
            (0..*num).into_iter().for_each(|_| {
                let cq = Qubit::new(QubitKind::Comm, counter);
                q_to_partner.insert(cq.id(), partner.clone());
                let initial_cost = Default::default();
                h.push(Reverse((initial_cost, cq)));
                counter += 1;
            });
            cq.insert(partner.clone(), h);
        });
        Self {
            q,
            cq,
            q_to_partner,
            used_q: HashSet::new(),
            used_cq: HashSet::new(),
        }
    }

    pub fn init_data_qubit(&mut self) -> Option<(Qubit, EvaluationCost)> {
        if let Some(Reverse((cost, q))) = self.q.pop() {
            self.used_q.insert(q.id());
            Some((q, cost))
        } else {
            None
        }
    }

    pub fn init_comm_qubit(&mut self, partner: ParticipantId) -> Option<(Qubit, EvaluationCost)> {
        if let Some(Reverse((cost, q))) = self.cq.get_mut(&partner).unwrap().pop() {
            self.used_cq.insert(q.id());
            Some((q, cost))
        } else {
            None
        }
    }

    pub fn free_qubit(&mut self, q: Qubit, cost: EvaluationCost) {
        match q.kind() {
            QubitKind::Data => {
                assert!(self.used_q.contains(&q.id()));
                self.used_q.remove(&q.id());
                self.q.push(Reverse((cost, q)));
            },
            QubitKind::Comm => {
                assert!(self.used_cq.contains(&q.id()));
                self.used_cq.remove(&q.id());
                let partner = self.q_to_partner[&q.id()];
                self.cq.get_mut(&partner).unwrap().push(Reverse((cost, q)));
            },
        }
    }

    pub fn debug_print(&self) {
        println!("{:?}", self.q);
        println!("{:?}", self.cq);
        println!("{:?}", self.used_q);
        println!("{:?}", self.used_cq);
    }
}

pub struct Participant {
    /// The self ID.
    id: ParticipantId,
    reg: Registers,
    processes: Vec<Process>,
    current_proc_idx: usize,
    shared_memory: Rc<RefCell<SharedMemory>>,
    cost_when_finished: HashMap<String, EvaluationCost>,
    var_to_qubit: HashMap<String, Qubit>,
    latency: Latency,
    issue_timestamp: Vec<(u64, usize)>, // [(time, process_idx)]
}

impl Participant {
    pub fn new(id: ParticipantId, num_q: usize, num_cq: HashMap<ParticipantId, u32>, mem: Rc<RefCell<SharedMemory>>, latency: Latency) -> Self {
        Self {
            id,
            reg: Registers::new(num_q, num_cq),
            processes: Vec::new(),
            current_proc_idx: 0,
            shared_memory: mem,
            cost_when_finished: HashMap::new(),
            var_to_qubit: HashMap::new(),
            latency,
            issue_timestamp: Vec::new(),
        }
    }

    pub fn id(&self) -> ParticipantId {
        self.id
    }

    pub fn process_size(&self) -> usize {
        self.processes.len()
    }

    pub fn current_process(&self) -> usize {
        self.current_proc_idx
    }

    pub fn issue_timestamp(&self) -> &Vec<(u64, usize)> {
        &self.issue_timestamp
    }

    pub fn add_process(&mut self, process: Vec<Process>) {
        self.processes = process;
    }

    pub fn advance(&mut self) -> u32 {
        let start = self.current_proc_idx;
        while self.current_proc_idx < self.processes.len() {
            let idx = self.current_proc_idx;
            if !self.try_issue(self.processes[idx].clone()) {
                break;
            }
            self.current_proc_idx += 1;
        }
        (self.current_proc_idx - start) as u32
    }

    fn try_issue(&mut self, process: Process) -> bool {
        let latency = self.latency.latency(&process);
        let issued_time = match process {
            Process::Open(proc) => {
                let mut mem = self.shared_memory.borrow_mut();
                mem.open_session(proc.id);
                Some(0) // TODO
            },
            Process::Init(proc) => {
                let x = proc.dst.clone();
                if let Some((q, mut cost)) = self.reg.init_data_qubit() {
                    self.var_to_qubit.insert(x.clone(), q);
                    let issued_time = cost.total_time();
                    cost.add_total_time(latency);
                    self.cost_when_finished.insert(x, cost);
                    Some(issued_time)
                } else {
                    None
                }
            },
            Process::Free(proc) => {
                let q = self.var_to_qubit.remove(&proc.arg).unwrap();
                let cost = self.cost_when_finished[&proc.arg];
                let issued_time = cost.total_time();
                self.reg.free_qubit(q, cost);
                Some(issued_time)
            },
            Process::GenEnt(proc) => {
                let x = proc.x.clone();
                let mut mem = self.shared_memory.borrow_mut();
                if let Some((q, cost)) = self.reg.init_comm_qubit(proc.p) {
                    mem.request_ent(proc.p, cost, proc.label.clone());
                    if let Some(cost2) = mem.check_ent(self.id, proc.label) {
                        let mut cost = collect_cost(vec![cost, cost2]);
                        let issued_time = cost.total_time();
                        self.var_to_qubit.insert(x.clone(), q);
                        cost.add_gen_ent_time(latency);
                        cost.add_e_depth(1);
                        self.cost_when_finished.insert(x, cost);
                        Some(issued_time)
                    } else { // wait for the partner
                        self.reg.free_qubit(q, cost);
                        None
                    }
                } else {
                    None
                }
            },
            Process::EntSwap(proc) => {
                let args = [&proc.arg1, &proc.arg2];
                let mut cost = collect_cost(args.iter().map(|&var| self.cost_when_finished[var]).collect());
                let issued_time = cost.total_time();
                cost.add_total_time(latency);
                args.iter().for_each(|&var| {
                    self.cost_when_finished.insert(var.clone(), cost);
                    self.reg.free_qubit(self.var_to_qubit.remove(var).unwrap(), cost);
                });
                [proc.x1, proc.x2].into_iter().for_each(|var| {
                    self.cost_when_finished.insert(var, cost);
                });
                Some(issued_time)
            },
            Process::Send(proc) => {
                let (l, e) = proc.data;
                let dummy_val = Value::Bool(true); // TODO
                let mut cost = collect_cost(inquir::variables(&e).into_iter().map(|var| {
                    self.cost_when_finished[&var]
                }).collect());
                let issued_time = cost.total_time();
                cost.add_total_time(latency);
                cost.add_c_depth(1);
                let send_data = SendData::new(l, cost, dummy_val);
                self.shared_memory.borrow_mut().send(proc.s, proc.dst, send_data);
                Some(issued_time)
            },
            Process::Recv(proc) => {
                let (l, var) = proc.data;
                if let Some(recv_data) = self.shared_memory.borrow_mut().recv(proc.s, self.id, l) {
                    let mut cost = recv_data.cost();
                    let issued_time = cost.total_time();
                    cost.add_total_time(latency);
                    cost.add_c_depth(1);
                    self.cost_when_finished.insert(var, cost);
                    Some(issued_time)
                } else {
                    None
                }
            },
            Process::Apply(proc) => {
                let qs_cost = collect_cost(proc.args.iter().map(|var| self.cost_when_finished[var]).collect());
                let ctrl_cost = proc.ctrl.map_or(Default::default(), |e| {
                    collect_cost(inquir::variables(&e).into_iter().map(|var| self.cost_when_finished[&var]).collect())
                });
                let mut cost = collect_cost(vec![qs_cost, ctrl_cost]);
                let issued_time = cost.total_time();
                cost.add_total_time(latency);
                proc.args.into_iter().for_each(|var| {
                    *self.cost_when_finished.get_mut(&var).unwrap() = cost;
                });
                Some(issued_time)
            },
            Process::Measure(proc) => {
                let prev_costs = proc.args.iter().map(|var| self.cost_when_finished[var]).collect();
                let mut cost = collect_cost(prev_costs);
                let issued_time = cost.total_time();
                cost.add_total_time(latency);
                proc.args.into_iter().for_each(|var| {
                    *self.cost_when_finished.get_mut(&var).unwrap() = cost;
                });
                self.cost_when_finished.insert(proc.dst, cost);
                Some(issued_time)
            },
            Process::QSend(_) => unimplemented!(),
            Process::QRecv(_) => unimplemented!(),
            Process::RCXC(_) => unimplemented!(),
            Process::RCXT(_) => unimplemented!(),
            Process::Parallel(_) => unimplemented!(),
        };

        issued_time.map_or(false, |t| {
            self.issue_timestamp.push((t, self.current_proc_idx));
            true
        })
    }

    pub fn is_completed(&self) -> bool {
        self.processes.len() == self.current_proc_idx
    }

    pub fn evaluation_cost(&self) -> EvaluationCost {
        let costs = self.cost_when_finished.iter().map(|(_, cost)| *cost).collect();
        collect_cost(costs)
    }

    pub fn debug_print(&self) {
        if self.is_completed() {
            println!("{}: complete.", self.id);
        } else {
            println!("{}: {}", self.id, self.processes[self.current_proc_idx]);
            self.reg.debug_print();
        }
    }
}
