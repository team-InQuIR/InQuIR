use crate::simulation::{
    participant::Participant,
    shared_memory::SharedMemory,
    evaluation_cost::{EvaluationCost, collect_cost},
    latency::Latency,
};
use crate::arch::Configuration;
use inquir::{
    ParticipantId,
    System,
};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

/// Note: We do not calculate the quantum state because of its computational cost.
pub struct Simulator {
    #[allow(unused)]
    shared_memory: Rc<RefCell<SharedMemory>>,
    participants: Vec<Participant>,
    mp: MultiProgress,
    pbs: Vec<ProgressBar>,
}

impl Simulator {
    pub fn new(s: &System, config: &Configuration) -> Self {
        let shared_memory = Rc::new(RefCell::new(SharedMemory::new()));
        let participants: Vec<_> = (0..config.node_size()).map(|i| {
            let id = ParticipantId::new(i as u32);
            let g = config.connections();
            let num_q = config.node_info_ref(i).num_of_qubits() as usize;
            let mut num_cq = HashMap::new();
            g.outgoing_edges(i).iter().for_each(|&eidx| {
                let e = g.edge(eidx);
                let p = ParticipantId::new(e.target() as u32);
                num_cq.insert(p, *e.weight());
            });
            // TODO
            g.incoming_edges(i).iter().for_each(|&eidx| {
                let e = g.edge(eidx);
                let p = ParticipantId::new(e.source() as u32);
                num_cq.insert(p, *e.weight());
            });
            let process = inquir::system::projection(s, id).unwrap();
            let latency = Latency::new(config.node_info_ref(id.to_usize()).clone());
            let mut p = Participant::new(id, num_q, num_cq, Rc::clone(&shared_memory), latency);
            p.add_process(process);
            p
        }).collect();

        let mp = MultiProgress::new();
        let sty = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:70.cyan/blue} {pos:>7} {msg}"
        ).unwrap().progress_chars("##-");
        let mut pbs = Vec::new();
        pbs.push(mp.add(ProgressBar::new(participants[0].process_size() as u64)));
        for i in 1..participants.len() {
            pbs.push(mp.insert_after(&pbs[i - 1], ProgressBar::new(participants[i].process_size() as u64)));
        }
        pbs.iter().for_each(|bar| bar.set_style(sty.clone()));

        Self {
            shared_memory,
            participants,
            mp,
            pbs,
        }
    }

    pub fn run(mut self) -> EvaluationCost {
        println!("Start simulation.");
        while self.participants.iter().any(|p| !p.is_completed()) {
            let steps: Vec<_> = self.participants.iter_mut().map(|p| {
                let steps = p.advance();

                let msg = format!("[{}/{}]", p.current_process(), p.process_size());
                self.pbs[p.id().to_usize()].set_position(p.current_process() as u64);
                self.pbs[p.id().to_usize()].set_message(msg);

                steps
            }).collect();

            if steps.into_iter().all(|n| n == 0) {
                self.participants.iter().for_each(|p| p.debug_print());
                panic!("Simulation got stuck!"); // TODO
            }
        }
        self.mp.clear().unwrap();
        collect_cost(self.participants.into_iter().map(|p| p.evaluation_cost()).collect())
    }
}

