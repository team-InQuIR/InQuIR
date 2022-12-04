use crate::graph::{DiGraph, NodeIndex};
use std::collections::VecDeque;

/// Returns the topological order of the given directed acyclic graph.
pub fn toposort<N: Clone, E: Clone>(g: &DiGraph<N, E>) -> Option<Vec<NodeIndex>> {
    let n = g.node_count();
    let mut in_deg: Vec<_> = (0..n).into_iter().map(|i| g.incoming_edges(i).len()).collect();
    let mut s = VecDeque::new();
    let mut res = Vec::new();
    in_deg.iter().enumerate().for_each(|(i, &d)| if d == 0 { s.push_back(i); });
    while let Some(v) = s.pop_front() {
        res.push(v);
        g.neighbors(v).iter().for_each(|&u| {
            in_deg[u] = in_deg[u] - 1;
            if in_deg[u] == 0 {
                s.push_back(u);
            }
        });
    }
    if in_deg.iter().all(|&d| d == 0) {
        Some(res)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::DiGraph;
    use super::toposort;

    #[test]
    fn test_toposort() {
        let mut g = DiGraph::new();
        g.add_node(());
        g.add_node(());
        g.add_node(());
        g.add_node(());
        g.add_edge(0, 2, ());
        g.add_edge(1, 2, ());
        g.add_edge(2, 3, ());
        let ord = toposort(&g).unwrap();
        let b1 = ord == vec![0, 1, 2, 3];
        let b2 = ord == vec![1, 0, 2, 3];
        dbg!(ord);
        assert!(b1 || b2);
    }
}

