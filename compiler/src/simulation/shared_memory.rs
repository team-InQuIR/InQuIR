use inquir::{
    SessionId,
    ParticipantId,
    Label,
    Value,
};
use crate::simulation::comm_buffer::{SendData, CommBuffer};
use std::collections::HashMap;

pub struct SharedMemory {
    ent_requests: Vec<Vec<(u32, Label)>>,
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

    pub fn request_ent(&mut self, partner: ParticipantId, t: u32, l: Label) {
        let partner = partner.to_usize();
        while self.ent_requests.len() <= partner {
            self.ent_requests.push(Vec::new());
        }
        self.ent_requests[partner].push((t, l));
    }

    pub fn check_ent(&mut self, p: ParticipantId, l: Label) -> Option<u32> {
        let p = p.to_usize();
        while self.ent_requests.len() <= p {
            self.ent_requests.push(Vec::new());
        }
        if let Some(idx) = self.ent_requests[p].iter().position(|(_, l2)| l == *l2) {
            let (t, _) = self.ent_requests[p].remove(idx);
            Some(t)
        } else {
            None
        }
    }

    pub fn send(&mut self, s: SessionId, p: ParticipantId, t: u32, l: Label, v: Value) {
        let p = p.to_usize();
        while self.heap[&s].len() <= p {
            self.heap.get_mut(&s).unwrap().push(CommBuffer::new());
        }
        self.heap.get_mut(&s).unwrap()[p].push(SendData::new(t, l, v));
    }

    pub fn recv(&mut self, s: SessionId, p: ParticipantId, l: Label) -> Option<(u32, Value)> {
        let p = p.to_usize();
        while self.heap[&s].len() <= p {
            self.heap.get_mut(&s).unwrap().push(CommBuffer::new());
        }
        self.heap.get_mut(&s).unwrap()[p].pop(l)
    }
}
