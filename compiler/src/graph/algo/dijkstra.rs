use super::super::{Graph, DiGraph, Node, NodeIndex, EdgeIndex, EdgeType};
use std::collections::BinaryHeap;

// TODO: generalize the type of edge weights
/// Returns a pair of (dist, prev), where
///   * dist[i] := the weight of the s--i shortest path
///   * prev[i] := the previous node in the shortest graph
pub fn dijkstra<N: Clone, E: Clone, ET: EdgeType, F>(
    g: &Graph<N, E, ET>,
    weight: F,
    s: NodeIndex
) -> (Vec<Option<u32>>, Vec<Option<NodeIndex>>) where F: Fn(EdgeIndex) -> u32 {
    let node_size = g.node_count();
    let mut dist = vec![None; node_size];
    let mut prev = vec![None; node_size];

    let mut que = BinaryHeap::new();
    que.push((0, s));
    dist[s] = Some(0);
    while que.len() > 0 {
        let (d, v) = que.pop().unwrap();
        if dist[v].map_or(false, |d2| d > d2) {
            continue;
        }
        for &eidx in g.outgoing_edges(v) {
            let to = g.edge(eidx).target();
            let nd = d + weight(eidx);
            if dist[to].map_or(true, |d2| nd < d2) {
                dist[to] = Some(nd);
                prev[to] = Some(v);
                que.push((nd, to));
            }
        }
        // TODO
        if !ET::is_directed() {
            for &eidx in g.incoming_edges(v) {
                let to = g.edge(eidx).source(); // reverse direction
                let nd = d + weight(eidx);
                if dist[to].map_or(true, |d2| nd < d2) {
                    dist[to] = Some(nd);
                    prev[to] = Some(v);
                    que.push((nd, to));
                }
            }
        }
    }

    (dist, prev)
}


#[cfg(test)]
mod test {
    use super::super::super::{DiGraph, UnGraph};
    use super::dijkstra;

    #[test]
    fn test_dijkstra1() {
        let mut g = DiGraph::new();
        let v1 = g.add_node(());
        let v2 = g.add_node(());
        let v3 = g.add_node(());
        let v4 = g.add_node(());
        let v5 = g.add_node(());
        g.add_edge(v1, v2, 4);
        g.add_edge(v1, v3, 1);
        g.add_edge(v2, v4, 2);
        g.add_edge(v2, v5, 1);
        g.add_edge(v3, v2, 2);
        g.add_edge(v3, v4, 3);
        g.add_edge(v4, v5, 2);
        let (dist, prev) = dijkstra(&g, |eid| *g.edge(eid).weight(), 0);
        assert_eq!(dist[0], Some(0));
        assert_eq!(dist[1], Some(3));
        assert_eq!(dist[2], Some(1));
        assert_eq!(dist[3], Some(4));
        assert_eq!(dist[4], Some(4));
        assert_eq!(prev[0], None);
        assert_eq!(prev[1], Some(2));
        assert_eq!(prev[2], Some(0));
        assert_eq!(prev[3], Some(2));
        assert_eq!(prev[4], Some(1));
    }

    #[test]
    fn test_dijkstra_undirected() {
        let mut g = UnGraph::new();
        let v1 = g.add_node(());
        let v2 = g.add_node(());
        let v3 = g.add_node(());
        let v4 = g.add_node(());
        let v5 = g.add_node(());
        g.add_edge(v1, v2, 4);
        g.add_edge(v1, v3, 1);
        g.add_edge(v2, v4, 2);
        g.add_edge(v2, v5, 1);
        g.add_edge(v3, v2, 2);
        g.add_edge(v3, v4, 3);
        g.add_edge(v4, v5, 2);
        let (dist, prev) = dijkstra(&g, |eid| *g.edge(eid).weight(), 1);
        assert_eq!(dist[0], Some(3));
        assert_eq!(dist[1], Some(0));
        assert_eq!(dist[2], Some(2));
        assert_eq!(dist[3], Some(2));
        assert_eq!(dist[4], Some(1));
        assert_eq!(prev[0], Some(2));
        assert_eq!(prev[1], None);
        assert_eq!(prev[2], Some(1));
        assert_eq!(prev[3], Some(1));
        assert_eq!(prev[4], Some(1));
    }
}
