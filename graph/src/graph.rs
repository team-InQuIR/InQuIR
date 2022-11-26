pub type NodeIndex = usize;
pub type EdgeIndex = usize;

use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Node<W: Clone> {
    weight: W,
    incoming: Vec<EdgeIndex>,
    outgoing: Vec<EdgeIndex>,
}

impl<W: Clone> Node<W> {
    pub fn new(weight: W) -> Self {
        Node {
            weight,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }

    pub fn weight(&self) -> &W {
        &self.weight
    }

    pub fn incoming(&self) -> &Vec<EdgeIndex> {
        &self.incoming
    }

    pub fn outgoing(&self) -> &Vec<EdgeIndex> {
        &self.outgoing
    }
}

#[derive(Debug, Clone)]
pub struct Edge<E: Clone> {
    source: NodeIndex,
    target: NodeIndex,
    weight: E,
}

impl<E: Clone> Edge<E> {
    pub fn new(source: NodeIndex, target: NodeIndex, weight: E) -> Self {
        Self {
            source,
            target,
            weight
        }
    }

    pub fn source(&self) -> NodeIndex {
        self.source
    }

    pub fn target(&self) -> NodeIndex {
        self.target
    }

    pub fn weight(&self) -> &E {
        &self.weight
    }

    pub fn weight_mut(&mut self) -> &mut E {
        &mut self.weight
    }
}

pub trait EdgeType {
    fn is_directed() -> bool;
}

#[derive(Debug, Clone)]
pub enum Directed {}
#[derive(Debug, Clone)]
pub enum Undirected {}

impl EdgeType for Directed {
    fn is_directed() -> bool {
        true
    }
}
impl EdgeType for Undirected {
    fn is_directed() -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone, ET: EdgeType> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
    _ety: PhantomData<fn () -> ET>,
}

impl<N: Clone, E: Clone, ET: EdgeType> Graph<N, E, ET> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            _ety: PhantomData,
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, u: NodeIndex) -> &Node<N> {
        &self.nodes[u]
    }

    pub fn nodes(&self) -> &Vec<Node<N>> {
        &self.nodes
    }

    pub fn edge(&self, eid: EdgeIndex) -> &Edge<E> {
        &self.edges[eid]
    }

    pub fn edge_weight_mut(&mut self, eid: EdgeIndex) -> &mut E {
        self.edges[eid].weight_mut()
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex {
        let node = Node::new(weight);
        let id = self.node_count();
        self.nodes.push(node);
        id
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, weight: E) -> EdgeIndex {
        let eid = self.edges.len();
        self.edges.push(Edge::new(source, target, weight));
        self.nodes[source].outgoing.push(eid);
        self.nodes[target].incoming.push(eid);
        eid
    }

    pub fn neighbors(&self, u: NodeIndex) -> Vec<NodeIndex> {
        // TODO: inefficient because of copy
        if ET::is_directed() {
            self.nodes[u].outgoing.iter().map(|&eid| self.edges[eid].target()).collect()
        } else {
            self.nodes[u].outgoing.iter().map(|&eid| self.edges[eid].target())
                .chain(self.nodes[u].incoming.iter().map(|&eid| self.edges[eid].source()))
                .collect()
        }
    }

    pub fn incoming_edges(&self, u: NodeIndex) -> &Vec<EdgeIndex> {
        &self.nodes[u].incoming
    }

    pub fn outgoing_edges(&self, u: NodeIndex) -> &Vec<EdgeIndex> {
        &self.nodes[u].outgoing
    }
}

pub type DiGraph<N, E> = Graph<N, E, Directed>;
pub type UnGraph<N, E> = Graph<N, E, Undirected>;
