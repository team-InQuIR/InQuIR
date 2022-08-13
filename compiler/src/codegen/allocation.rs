use std::collections::HashMap;
use crate::arch::Configuration;
use crate::hir;

pub enum RemoteOp {
    RCX,
    Move(Vec<(String, u32, u32)>),
}

pub trait NodeAllocator {
    fn current_pos(&self, id: &String) -> u32;
    fn next(&mut self, id1: &String, id2: &String) -> RemoteOp;
}

pub struct NaiveNodeAllocator {
    current_pos: HashMap<String, u32>,
    free_qubits: Vec<u32>,
}

impl NaiveNodeAllocator {
    pub fn new(exps: &Vec<hir::Expr>, config: &Configuration) -> Self {
        let (current_pos, free_qubits) = NaiveNodeAllocator::create_initial_map(&exps, &config);
        Self {
            current_pos,
            free_qubits,
        }
    }

    fn create_initial_map(exps: &Vec<hir::Expr>, config: &Configuration) -> (HashMap<String, u32>, Vec<u32>) {
        let mut num_of_qubits: Vec<u32> = (0..config.node_size()).map(|i| config.node_info_ref(i).num_of_qubits()).collect();
        let mut map = HashMap::new();
        let mut target_node = 0;
        let mut next_node = || {
            while target_node < num_of_qubits.len() && num_of_qubits[target_node] == 0 {
                target_node += 1;
            }
            if target_node >= num_of_qubits.len() {
                panic!("target_node >= node_of_qubits.len()");
            }
            num_of_qubits[target_node as usize] -= 1;
            target_node
        };
        for e in exps {
            match e {
                hir::Expr::Init(hir::InitExpr { dst }) => {
                    assert!(!map.contains_key(dst));
                    let location = next_node() as u32;
                    map.insert(dst.clone(), location);
                },
                _ => {},
            }
        }
        (map, num_of_qubits)
    }
}

impl NodeAllocator for NaiveNodeAllocator {
    fn current_pos(&self, id: &String) -> u32 {
        self.current_pos[id]
    }

    fn next(&mut self, id1: &String, id2: &String) -> RemoteOp {
        if !self.current_pos.contains_key(id1) || !self.current_pos.contains_key(id2) {
            panic!("NaiveNodeAllocator::next");
        }
        let pos1 = *self.current_pos.get(id1).unwrap();
        let pos2 = *self.current_pos.get(id2).unwrap();
        if pos1 == pos2 { // local operation
            return RemoteOp::Move(Vec::new());
        }
        if self.free_qubits[pos1 as usize] > 0 {
            // move to pos1
            self.free_qubits[pos1 as usize] -= 1;
            self.free_qubits[pos2 as usize] += 1;
            *self.current_pos.get_mut(id2).unwrap() = pos1;
            return RemoteOp::Move(vec![(id2.clone(), pos2, pos1)]);
        }
        if self.free_qubits[pos2 as usize] > 0 {
            self.free_qubits[pos2 as usize] -= 1;
            self.free_qubits[pos1 as usize] += 1;
            *self.current_pos.get_mut(id1).unwrap() = pos2;
            return RemoteOp::Move(vec![(id1.clone(), pos1, pos2)]);
        }
        // find another position
        let mut tmp_node = None;
        for i in 0..self.free_qubits.len() {
            if self.free_qubits[i] > 0 {
                tmp_node = Some(i);
            }
        }
        let tmp_node = tmp_node.expect("There is no space for sending a qubit") as u32;
        let mut res = Vec::new();
        for (k, pos) in &self.current_pos {
            if *pos == pos1 && k != id1 {
                // k: pos1 --> tmp_node
                // id2: pos2 --> pos1
                res.push((k.clone(), pos1, tmp_node));
                res.push((id2.clone(), pos2, pos1));
                self.free_qubits[pos2 as usize] += 1;
                self.free_qubits[tmp_node as usize] -= 1;
                break;
            }
            if *pos == pos2 && k != id2 {
                // k: pos2 --> tmp_node
                // id1: pos1 --> pos2
                res.push((k.clone(), pos2, tmp_node));
                res.push((id1.clone(), pos1, pos2));
                self.free_qubits[pos1 as usize] += 1;
                self.free_qubits[tmp_node as usize] -= 1;
                break;
            }
        }
        assert!(res.len() > 0);
        for (id, _, to) in &res {
            *self.current_pos.get_mut(id).unwrap() = *to;
        }
        RemoteOp::Move(res)
    }
}
