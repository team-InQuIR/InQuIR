use inquir::{
    ParticipantId,
    Qubit, QubitKind,
    Value,
    Process,
};
use crate::simulation::shared_memory::SharedMemory;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;

#[derive(Debug, Clone)]
struct Registers {
    q: BinaryHeap<Reverse<(u32, Qubit)>>,
    cq: HashMap<ParticipantId, BinaryHeap<Reverse<(u32, Qubit)>>>,
    q_to_partner: HashMap<u32, ParticipantId>,
    used_q: HashSet<u32>,
    used_cq: HashSet<u32>,
}

impl Registers {
    pub fn new(num_q: usize, num_cq: HashMap<ParticipantId, u32>) -> Self {
        let q = (0..num_q).fold(BinaryHeap::new(), |mut h, i| {
            let q = Qubit::new(QubitKind::Data, i as u32);
            h.push(Reverse((0, q)));
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
                h.push(Reverse((0, cq)));
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

    pub fn init_data_qubit(&mut self) -> Option<(Qubit, u32)> {
        if let Some(Reverse((t, q))) = self.q.pop() {
            self.used_q.insert(q.id());
            Some((q, t))
        } else {
            None
        }
    }

    pub fn init_comm_qubit(&mut self, partner: ParticipantId) -> Option<(Qubit, u32)> {
        if let Some(Reverse((t, q))) = self.cq.get_mut(&partner).unwrap().pop() {
            self.used_cq.insert(q.id());
            Some((q, t))
        } else {
            None
        }
    }

    pub fn free_qubit(&mut self, q: Qubit, t: u32) {
        match q.kind() {
            QubitKind::Data => {
                assert!(self.used_q.contains(&q.id()));
                self.used_q.remove(&q.id());
                self.q.push(Reverse((t, q)));
            },
            QubitKind::Comm => {
                assert!(self.used_cq.contains(&q.id()));
                self.used_cq.remove(&q.id());
                let partner = self.q_to_partner[&q.id()];
                self.cq.get_mut(&partner).unwrap().push(Reverse((t, q)));
            },
        }
    }
}

pub struct Participant {
    /// The self ID.
    id: ParticipantId,
    reg: Registers,
    processes: Vec<Process>,
    current_proc_idx: usize,
    shared_memory: Rc<RefCell<SharedMemory>>,
    time_until: HashMap<String, u32>,
    var_to_qubit: HashMap<String, Qubit>,
}

impl Participant {
    pub fn new(id: ParticipantId, num_q: usize, num_cq: HashMap<ParticipantId, u32>, mem: Rc<RefCell<SharedMemory>>) -> Self {
        Self {
            id,
            reg: Registers::new(num_q, num_cq),
            processes: Vec::new(),
            current_proc_idx: 0,
            shared_memory: mem,
            time_until: HashMap::new(),
            var_to_qubit: HashMap::new(),
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
        let latency = self.latency(&process);
        match process {
            Process::Open(proc) => {
                let mut mem = self.shared_memory.borrow_mut();
                mem.open_session(proc.id);
                true
            },
            Process::Init(proc) => {
                let x = proc.dst.clone();
                if let Some((q, t)) = self.reg.init_data_qubit() {
                    self.var_to_qubit.insert(x.clone(), q);
                    self.time_until.insert(x, t + latency);
                    true
                } else {
                    false
                }
            },
            Process::Free(proc) => {
                let q = self.var_to_qubit.remove(&proc.arg).unwrap();
                let t = self.time_until[&proc.arg];
                self.reg.free_qubit(q, t);
                true
            },
            Process::GenEnt(proc) => {
                let x = proc.x.clone();
                let mut mem = self.shared_memory.borrow_mut();
                if let Some((q, t)) = self.reg.init_comm_qubit(proc.p) {
                    mem.request_ent(proc.p, t, proc.label.clone());
                    if let Some(t2) = mem.check_ent(self.id, proc.label) {
                        let t = u32::max(t, t2);
                        self.var_to_qubit.insert(x.clone(), q);
                        self.time_until.insert(x, t + latency);
                        true
                    } else { // wait for the partner
                        self.reg.free_qubit(q, t);
                        false
                    }
                } else {
                    false
                }
            },
            Process::EntSwap(proc) => {
                let args = [&proc.arg1, &proc.arg2];
                let s_time = args.iter().map(|&var| self.time_until[var]).max().unwrap();
                let e_time = s_time + latency;
                args.iter().for_each(|&var| {
                    self.time_until.insert(var.clone(), e_time);
                    self.reg.free_qubit(self.var_to_qubit.remove(var).unwrap(), e_time);
                });
                [proc.x1, proc.x2].into_iter().for_each(|var| {
                    self.time_until.insert(var, e_time);
                });
                true
            },
            Process::Send(proc) => {
                let (l, e) = proc.data;
                let dummy_val = Value::Bool(true); // TODO
                let s_time = inquir::variables(&e).into_iter().map(|var| {
                    self.time_until[&var]
                }).max().unwrap();
                self.shared_memory.borrow_mut().send(proc.s, proc.dst, s_time + latency, l, dummy_val);
                true
            },
            Process::Recv(proc) => {
                let (l, var) = proc.data;
                if let Some((recv_t, _)) = self.shared_memory.borrow_mut().recv(proc.s, self.id, l) {
                    let e_time = recv_t + latency;
                    self.time_until.insert(var, e_time);
                    true
                } else {
                    false
                }
            },
            Process::Apply(proc) => {
                let qs_t = proc.args.iter().map(|var| self.time_until[var]).max().unwrap();
                let ctrl_t = proc.ctrl.map_or(0, |e| {
                    inquir::variables(&e).into_iter().map(|var| self.time_until[&var]).max().map_or(0, |t| t)
                });
                let dep_t = u32::max(qs_t, ctrl_t);
                let e_time = dep_t + latency;
                proc.args.into_iter().for_each(|var| {
                    *self.time_until.get_mut(&var).unwrap() = e_time;
                });
                true
            },
            Process::Measure(proc) => {
                let s_time = proc.args.iter().map(|var| self.time_until[var]).max().unwrap();
                let e_time = s_time + latency;
                proc.args.into_iter().for_each(|var| {
                    *self.time_until.get_mut(&var).unwrap() = e_time;
                });
                self.time_until.insert(proc.dst, e_time);
                true
            },
            Process::QSend(_) => unimplemented!(),
            Process::QRecv(_) => unimplemented!(),
            Process::RCXC(_) => unimplemented!(),
            Process::RCXT(_) => unimplemented!(),
            Process::Parallel(_) => unimplemented!(),
        }
    }

    fn latency(&self, process: &Process) -> u32 { // TODO: make customizable
        match process {
            Process::Open(_) => 0,
            Process::GenEnt(_) => 10,
            Process::EntSwap(_) => 3,
            // These operations must be decomposed before simulation.
            Process::Parallel(_) => unimplemented!(),
            Process::QSend(_) => unimplemented!(),
            Process::QRecv(_) => unimplemented!(),
            Process::RCXC(_) => unimplemented!(),
            Process::RCXT(_) => unimplemented!(),
            _ => 1,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.processes.len() == self.current_proc_idx
    }

    pub fn current_time(&self) -> u32 {
        self.time_until.iter().map(|(_, t)| t).max().map_or(0, |t| *t)
    }
}
