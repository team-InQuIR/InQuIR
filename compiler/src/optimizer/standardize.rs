use inquir::{
    Process, ApplyProc,
    System,
    PrimitiveGate,
    Expr, BinOp,
};
use crate::{
    arch::Configuration,
    dependency_graph::DependencyGraphBuilder,
    codegen::convert_graph::convert_graph,
};
use std::time::Instant;

pub fn standardize(s: System, config: &Configuration) -> System {
    let builder = DependencyGraphBuilder::new();
    let mut g = builder.build(s);
    //println!("{}", g.as_graphviz());

    let time = Instant::now();
    let timeout = 10; // 10sec
    // TODO: more efficient program
    let mut updated = true;
    while updated && time.elapsed().as_secs() <= timeout {
        updated = false;
        let mut cur_node_idx = 0;
        while cur_node_idx < g.node_count() { // The number of nodes can be increased.
            let node1  = cur_node_idx;
            cur_node_idx += 1;
            if !g.node(node1).weight().1.is_app() {
                continue;
            }
            let to_nodes: Vec<_> = g.outgoing_edges(node1).iter().map(|&id| g.edge(id).target()).collect();
            for node2 in to_nodes {
                let (p, e1) = g.node(node1).weight();
                let (_, e2) = g.node(node2).weight();
                let p = *p;
                if e1.is_app() && e2.is_app() {
                    let app1 = e1.as_app().unwrap();
                    let app2 = e2.as_app().unwrap();
                    match (&app1.gate, &app2.gate) {
                        (_, PrimitiveGate::I) => {
                            if app1.gate != PrimitiveGate::I && app1.args.len() == 1 {
                                assert!(app1.args == app2.args);
                                g.swap_adjacent(node1, node2);
                                updated = true;
                            }
                        },
                        (PrimitiveGate::Z, PrimitiveGate::Z)
                        | (PrimitiveGate::X, PrimitiveGate::X)
                        | (PrimitiveGate::H, PrimitiveGate::H)
                        | (PrimitiveGate::T, PrimitiveGate::T) => {
                            assert!(app1.args == app2.args);
                            g.remove_node(node1);
                            let app = merge_app(app1, app2);
                            g.replace_stmt(node2, Process::Apply(app));
                            g.propagate_classical_deps(node1, node2);
                            updated = true;
                        },
                        (PrimitiveGate::X, PrimitiveGate::CX) => {
                            assert!(app2.ctrl == None);
                            if app1.args[0] == app2.args[0] { // X(1);CX(1,2) = CX(1,2);X(1);X(2)
                                g.swap_adjacent(node1, node2);
                                let app2 = ApplyProc {
                                    gate: PrimitiveGate::X,
                                    args: vec![app2.args[1].clone()],
                                    ctrl: app1.ctrl.clone(),
                                };
                                let node3 = g.insert_app_after(node2, p, app2);
                                g.propagate_classical_deps(node1, node3);
                            } else { // X(2);CX(1,2) = CX(1,2);X(2)
                                g.swap_adjacent(node1, node2);
                            }
                            updated = true;
                        },
                        (PrimitiveGate::Z, PrimitiveGate::CX) => {
                            assert!(app2.ctrl == None);
                            if app1.args[0] == app2.args[1] {
                                g.swap_adjacent(node1, node2);
                                let app2 = ApplyProc {
                                    gate: PrimitiveGate::Z,
                                    args: vec![app2.args[0].clone()],
                                    ctrl: app1.ctrl.clone(),
                                };
                                let node3 = g.insert_app_after(node2, p, app2);
                                g.propagate_classical_deps(node1, node3);
                            } else {
                                g.swap_adjacent(node1, node2);
                            }
                            updated = true;
                        },
                        (PrimitiveGate::Z, PrimitiveGate::H) => {
                            assert!(app1.args == app2.args);
                            g.replace_gate(node1, PrimitiveGate::X);
                            g.swap_adjacent(node1, node2);
                            updated = true;
                        },
                        (PrimitiveGate::X, PrimitiveGate::H) => {
                            assert!(app1.args == app2.args);
                            g.replace_gate(node1, PrimitiveGate::Z);
                            g.swap_adjacent(node1, node2);
                            updated = true;
                        },
                        (PrimitiveGate::Z, PrimitiveGate::T) => {
                            assert!(app1.args == app2.args);
                            g.swap_adjacent(node1, node2);
                            updated = true;
                        },
                        _ => {}
                    }
                } else if e1.is_app() && e2.is_measure() {
                    continue; // Disable temporarily
                    let app1 = e1.as_app().unwrap();
                    let meas2 = e2.as_measure().unwrap();
                    // 1. Does not support multi-qubit measurements
                    // 2. Does not propagate immediately before the final measurements
                    if meas2.args.len() > 1 || g.outgoing_edges(node2).len() == 0 {
                        continue;
                    }
                    match &app1.gate {
                        PrimitiveGate::X => {
                            g.remove_node(node1);
                            let ctrl = if let Some(ctrl) = app1.ctrl {
                                Expr::BinOp(BinOp::Xor, Box::new(Expr::Var(meas2.dst.clone())), Box::new(ctrl))
                            } else {
                                Expr::Not(Box::new(Expr::Var(meas2.dst.clone())))
                            };
                            g.replace_bexp_until_end(node2, &meas2.dst, ctrl);
                            updated = true;
                        },
                        PrimitiveGate::Z => {
                            g.remove_node(node1);
                            updated = true;
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    convert_graph(g, config)
}

fn merge_app(app1: ApplyProc, app2: ApplyProc) -> ApplyProc {
    assert!(app1.args == app2.args);
    assert!(app1.gate == app2.gate);
    let b1 = app1.ctrl;
    let b2 = app2.ctrl;
    match (b1, b2) {
        (None, None) => ApplyProc { gate: PrimitiveGate::I, args: app1.args, ctrl: None },
        (None, Some(b)) | (Some(b), None) => {
            let ctrl = Some(Expr::Not(Box::new(b)));
            ApplyProc { gate: app1.gate, args: app1.args, ctrl }
        },
        (Some(b1), Some(b2)) => {
            let ctrl = Some(Expr::BinOp(BinOp::Xor, Box::new(b1), Box::new(b2)));
            ApplyProc { gate: app1.gate, args: app1.args, ctrl }
        },
    }
}
