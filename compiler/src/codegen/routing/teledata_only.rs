use std::collections::BTreeMap;
use crate::hir;
use crate::arch::Configuration;
use crate::codegen::routing::{RemoteOpRouter, RemoteOp};

pub struct TeledataOnly {
    current_pos: BTreeMap<String, u32>,
    free_qubits: Vec<u32>,
}

impl TeledataOnly {
    pub fn new(exps: &Vec<hir::Expr>, config: &Configuration) -> Self {
        let (current_pos, free_qubits) = TeledataOnly::create_initial_map(&exps, &config);
        Self {
            current_pos,
            free_qubits,
        }
    }

    fn create_initial_map(exps: &Vec<hir::Expr>, config: &Configuration) -> (BTreeMap<String, u32>, Vec<u32>) {
        let mut num_of_qubits: Vec<u32> = (0..config.node_size()).map(|i| config.node_info_ref(i).num_of_qubits()).collect();
        let mut map = BTreeMap::new();
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

impl RemoteOpRouter for TeledataOnly {
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
            RemoteOp::LocalCX
        } else if self.free_qubits[pos1 as usize] > 0 {
            // move to pos1
            self.free_qubits[pos1 as usize] -= 1;
            self.free_qubits[pos2 as usize] += 1;
            *self.current_pos.get_mut(id2).unwrap() = pos1;
            RemoteOp::Move(id2.clone(), pos2, pos1)
        } else if self.free_qubits[pos2 as usize] > 0 {
            self.free_qubits[pos2 as usize] -= 1;
            self.free_qubits[pos1 as usize] += 1;
            *self.current_pos.get_mut(id1).unwrap() = pos2;
            RemoteOp::Move(id1.clone(), pos1, pos2)
        } else { // Use swap (RCX * 3)
            *self.current_pos.get_mut(id1).unwrap() = pos2;
            *self.current_pos.get_mut(id2).unwrap() = pos1;
            RemoteOp::RSwap
        }
    }
}
