use inquir::{
    SessionId,
    ParticipantId,
    Label,
};
use crate::simulation::evaluation_cost::EvaluationCost;
use crate::simulation::comm_buffer::{SendData, CommBuffer};
use std::collections::HashMap;

pub struct SharedMemory {
    ent_requests: Vec<Vec<(EvaluationCost, Label)>>,
    heap: HashMap<SessionId, Vec<CommBuffer>>,
}

impl SharedMemory {
    pub fn new() -> Self {
        Self {
            ent_requests: Vec::new(),
            heap: HashMap::new(),
        }
    }

    pub fn open_session(&mut self, s: SessionId) {
        if !self.heap.contains_key(&s) {
            self.heap.insert(s, Vec::new());
        }
    }

    pub fn request_ent(&mut self, partner: ParticipantId, cost: EvaluationCost, l: Label) {
        let partner = partner.to_usize();
        while self.ent_requests.len() <= partner {
            self.ent_requests.push(Vec::new());
        }
        self.ent_requests[partner].push((cost, l));
    }

    pub fn check_ent(&mut self, p: ParticipantId, l: Label) -> Option<EvaluationCost> {
        let p = p.to_usize();
        while self.ent_requests.len() <= p {
            self.ent_requests.push(Vec::new());
        }
        if let Some(idx) = self.ent_requests[p].iter().position(|(_, l2)| l == *l2) {
            let (cost, _) = self.ent_requests[p].remove(idx);
            Some(cost)
        } else {
            None
        }
    }

    pub fn send(&mut self, s: SessionId, p: ParticipantId, data: SendData) {
        let p = p.to_usize();
        while self.heap[&s].len() <= p {
            self.heap.get_mut(&s).unwrap().push(CommBuffer::new());
        }
        self.heap.get_mut(&s).unwrap()[p].push(data);
    }

    pub fn recv(&mut self, s: SessionId, p: ParticipantId, l: Label) -> Option<SendData> {
        let p = p.to_usize();
        while self.heap[&s].len() <= p {
            self.heap.get_mut(&s).unwrap().push(CommBuffer::new());
        }
        self.heap.get_mut(&s).unwrap()[p].pop(l)
    }
}
