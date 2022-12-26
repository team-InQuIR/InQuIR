use std::fs;
use graph::graph::{UnGraph, NodeIndex};
use serde::{
    Deserialize,
    Deserializer,
};

pub type ConnectionGraph = UnGraph<(), u32>;

#[derive(Deserialize, Debug, Clone)]
pub struct NodeInfo {
    /// The number of available qubits
    num_of_qubits: u32,
    #[serde(default = "default_single_gate_cost")]
    single_gate_cost: u64,
    #[serde(default = "default_local_cx_cost")]
    local_cx_cost: u64,
    #[serde(default = "default_gen_ent_cost")]
    gen_ent_cost: u64,
    #[serde(default = "default_measure_cost")]
    measure_cost: u64,
    #[serde(default = "default_classical_comm_cost")]
    classical_comm_cost: u64,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde(deserialize_with = "from_graph")]
    connections: ConnectionGraph,
    nodes: Vec<NodeInfo>,
}

impl NodeInfo {
    pub fn num_of_qubits(&self) -> u32 {
        self.num_of_qubits
    }

    pub fn single_gate_cost(&self) -> u64 {
        self.single_gate_cost
    }

    pub fn local_cx_cost(&self) -> u64 {
        self.local_cx_cost
    }

    pub fn gen_ent_cost(&self) -> u64 {
        self.gen_ent_cost
    }

    pub fn measure_cost(&self) -> u64 {
        self.measure_cost
    }

    pub fn classical_comm_cost(&self) -> u64 {
        self.classical_comm_cost
    }
}

/// Configuration of a distributed system
impl Configuration {
    pub fn new(nodes: Vec<NodeInfo>) -> Self {
        Self {
            connections: UnGraph::new(),
            nodes
        }
    }

    pub fn from_json(path: String) -> Self {
        let json_str = fs::read_to_string(path).unwrap();
        serde_json::from_str(&json_str).unwrap()
    }

    pub fn node_size(&self) -> usize {
        self.nodes.len()
    }

    pub fn node_info_ref(&self, id: usize) -> &NodeInfo {
        &self.nodes[id]
    }

    pub fn connections(&self) -> &ConnectionGraph {
        &self.connections
    }
}

fn from_graph<'de, D>(deserializer: D) -> Result<ConnectionGraph, D::Error>
where
    D: Deserializer<'de>
{
    let edges: Vec<(u32, u32, u32)> = Deserialize::deserialize(deserializer)?;
    let node_count = edges.iter().fold(0, |acc, &(u, v, _)| u32::max(acc, u32::max(u, v))) + 1;
    let mut g = ConnectionGraph::new();
    for _ in 0..node_count {
        g.add_node(());
    }
    for (u, v, cap) in edges {
        g.add_edge(u as NodeIndex, v as NodeIndex, cap);
    }
    Ok(g)
}

/*
Sung, Y. et al. Realization of High-Fidelity CZ and $ZZ$-Free iSWAP Gates with a Tunable Coupler. Phys. Rev. X 11, 021058 (2021).
*/
fn default_single_gate_cost() -> u64 {
    30 // 30 [ns]
}

/*
Wei, K. X. et al. Hamiltonian Engineering with Multicolor Drives for Fast Entangling Gates and Quantum Crosstalk Cancellation. Phys. Rev. Lett. 129, 060501 (2022).
*/
fn default_local_cx_cost() -> u64 {
    60 // 60 [ns]
}

/*
Ang, J. et al. Architectures for Multinode Superconducting Quantum Computers. Preprint at https://doi.org/10.48550/arXiv.2212.06167 (2022).
*/
fn default_gen_ent_cost() -> u64 {
    1000 // 1 [micro sec]
}

/*
Sunada, Y. et al. Fast Readout and Reset of a Superconducting Qubit Coupled to a Resonator with an Intrinsic Purcell Filter. Phys. Rev. Appl. 17, 044016 (2022).
*/
fn default_measure_cost() -> u64 {
    240 // 240 [ns]
}

fn default_classical_comm_cost() -> u64 {
    30
}
