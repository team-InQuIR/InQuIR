use std::fs;
use graph::graph::{UnGraph, NodeIndex};
use serde::{
    de::Error,
    Deserialize,
    Deserializer,
};

pub type ConnectionGraph = UnGraph<(), u32>;

#[derive(Deserialize, Debug)]
pub struct NodeInfo {
    /// The number of available qubits
    num_of_qubits: u32,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde(deserialize_with = "from_graph")]
    connections: ConnectionGraph,
    nodes: Vec<NodeInfo>,
}

impl NodeInfo {
    pub fn new(num_of_qubits: u32) -> Self {
        Self {
            num_of_qubits
        }
    }

    pub fn num_of_qubits(&self) -> u32 {
        self.num_of_qubits
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
