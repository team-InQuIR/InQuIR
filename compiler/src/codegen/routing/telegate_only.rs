use std::collections::BTreeMap;
use crate::hir;
use crate::arch::Configuration;
use crate::codegen::routing::router::{RemoteOpRouter, RemoteOp};

pub struct TelegateOnly {
    current_pos: BTreeMap<String, u32>,
}

impl TelegateOnly {
    pub fn new(exps: &Vec<hir::Expr>, config: &Configuration) -> Self {
        let current_pos = Self::create_initial_map(&exps, &config);
        Self {
            current_pos,
        }
    }

    fn create_initial_map(exps: &Vec<hir::Expr>, config: &Configuration) -> BTreeMap<String, u32> {
        let mut num_of_qubits: Vec<u32> = (0..config.node_size()).map(|i| config.node_info_ref(i).num_of_qubits()).collect();
        let mut map = BTreeMap::new();
        let mut target_node = 0;
        let mut next_node = || {
            while target_node < num_of_qubits.len() && num_of_qubits[target_node] == 0 {
                target_node += 1;
            }
            if target_node >= num_of_qubits.len() {
                // TODO
                panic!("target_node >= node_of_qubits.len()");
            }
            num_of_qubits[target_node as usize] -= 1;
            target_node
        };
        for e in exps {
            match e {
                hir::Expr::Init(e) => {
                    assert!(!map.contains_key(&e.dst));
                    let location = next_node() as u32;
                    map.insert(e.dst.clone(), location);
                },
                _ => {},
            }
        }
        map
    }
}

impl RemoteOpRouter for TelegateOnly {
    fn current_pos(&self, id: &String) -> u32 {
        self.current_pos[id]
    }

    /// Always choose remote CX gate
    fn next(&mut self, x1: &String, x2: &String) -> RemoteOp {
        if self.current_pos(x1) == self.current_pos(x2) {
            RemoteOp::LocalCX
        } else {
            RemoteOp::RCX
        }
    }
}
