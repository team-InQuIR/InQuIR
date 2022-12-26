use inquir::{
    Process,
    PrimitiveGate,
};
use crate::arch::NodeInfo;

pub struct Latency {
    node_info: NodeInfo,
}

impl Latency {
    pub fn new(node_info: NodeInfo) -> Self {
        Self {
            node_info,
        }
    }

    pub fn latency(&self, proc: &Process) -> u64 {
        match proc {
            Process::GenEnt(_) => self.node_info.gen_ent_cost(),
            Process::Open(_) => self.node_info.classical_comm_cost(),
            Process::EntSwap(_) => self.node_info.single_gate_cost(),
            Process::Send(_) => self.node_info.classical_comm_cost(),
            Process::Recv(_) => self.node_info.classical_comm_cost(),
            Process::Apply(p) => self.latency_gate(&p.gate),
            Process::Measure(_) => self.node_info.measure_cost(),
            Process::QSend(_) => unimplemented!(),
            Process::QRecv(_) => unimplemented!(),
            Process::RCXT(_) => unimplemented!(),
            Process::RCXC(_) => unimplemented!(),
            Process::Parallel(_) => unimplemented!(),
            _ => self.node_info.single_gate_cost(),
        }
    }

    pub fn latency_gate(&self, gate: &PrimitiveGate) -> u64 {
        match gate {
            PrimitiveGate::CX => self.node_info.local_cx_cost(),
            PrimitiveGate::RCX => unimplemented!(),
            _ => self.node_info.single_gate_cost(),
        }
    }
}
