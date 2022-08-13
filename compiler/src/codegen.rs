pub mod allocation;
pub mod always_rcx;

use allocation::NodeAllocator;
use inquir;
use crate::{
    arch::{Configuration, configuration::ConnectionGraph},
    hir,
    graph::{
        NodeIndex,
        algo::dijkstra,
    },
};

fn build_all_pair_shortest_path(g: &ConnectionGraph) -> Vec<Vec<Option<NodeIndex>>> {
    let mut prevs = vec![Vec::new(); g.node_count()];
    for s in 0..g.node_count() {
        let (_, p) = dijkstra(g, |_| 1, s);
        prevs[s] = p;
    }

    prevs
}

fn construct_shortest_path(
    prev: &Vec<Option<NodeIndex>>,
    from: NodeIndex,
    to: NodeIndex
) -> Vec<NodeIndex> {
    let mut path = Vec::new();
    path.push(to);
    let mut current = to;
    while prev[current] != None {
        current = prev[current].unwrap();
        path.push(current);
    }
    assert!(current == from);

    path
}

pub fn codegen(
    exps: Vec<hir::Expr>,
    config: &Configuration,
    mut allocator: Box<dyn NodeAllocator>
) -> Vec<Vec<inquir::Expr>> {
    let mut fresh_ent_id_ = 0;
    let mut fresh_ent_id = || {
        let res = format!("ent{}", fresh_ent_id_);
        fresh_ent_id_ += 1;
        res
    };

    let mut res = vec![vec![]; config.node_size()];
    let prevs = build_all_pair_shortest_path(config.connections());
    for e in exps {
        match e {
            hir::Expr::Ret => {},
            hir::Expr::Init(e) => {
                let pos = allocator.current_pos(&e.dst);
                res[pos as usize].push(inquir::Expr::from(inquir::InitExpr { dst: e.dst }));
            },
            hir::Expr::Apply(e) => {
                let gate = e.gate;
                let args = e.args;
                match gate {
                    hir::PrimitiveGate::CX => {
                        match allocator.next(&args[0], &args[1]) {
                            allocation::RemoteOp::RCX => {
                                let pos1 = allocator.current_pos(&args[0]);
                                let pos2 = allocator.current_pos(&args[1]);
                                if pos1 == pos2 {
                                    res[pos1 as usize].push(
                                        inquir::Expr::from(inquir::ApplyExpr {
                                            gate: inquir::PrimitiveGate::CX,
                                            args: args.clone()
                                        }));
                                } else {
                                    let pos1 = pos1 as NodeIndex;
                                    let pos2 = pos2 as NodeIndex;
                                    let path = construct_shortest_path(&prevs[pos1], pos1, pos2);
                                    let ent_ids: Vec<_> = (0..path.len()*2-2).map(|_| fresh_ent_id()).collect();
                                    // generate entanglements
                                    for i in 0..path.len() {
                                        if i >= 1 {
                                            let dst = ent_ids[2*i-1].clone();
                                            let partner = path[i - 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { dst, partner }));
                                        }
                                        if i + 1 < path.len() {
                                            let dst = ent_ids[2*i].clone();
                                            let partner = path[i + 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { dst, partner }));
                                        }
                                    }
                                    // perform entanglement swapping
                                    for i in 1..path.len()-1 {
                                        let arg1 = ent_ids[2*i-1].clone();
                                        let arg2 = ent_ids[2*i].clone();
                                        res[path[i]].push(inquir::Expr::from(inquir::EntSwapExpr { arg1, arg2 }));
                                    }
                                    res[pos1].push(inquir::Expr::from(inquir::RCXCExpr {
                                        arg: args[0].clone(), ent: ent_ids[0].clone()
                                    }));
                                    res[pos2].push(inquir::Expr::from(inquir::RCXTExpr {
                                        arg: args[1].clone(),
                                        ent: ent_ids[ent_ids.len()-1].clone()
                                    }));
                                }
                            },
                            allocation::RemoteOp::Move(path) => {
                                for (id, from, to) in path {
                                    let from = from as usize;
                                    let to = to as usize;
                                    let path = construct_shortest_path(&prevs[from], from, to);
                                    let ent_ids: Vec<_> = (0..path.len()*2-2).map(|_| fresh_ent_id()).collect();
                                    // generate entanglements
                                    for i in 0..path.len() {
                                        if i >= 1 {
                                            let dst = ent_ids[2*i-1].clone();
                                            let partner = path[i - 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { dst, partner }));
                                        }
                                        if i + 1 < path.len() {
                                            let dst = ent_ids[2*i].clone();
                                            let partner = path[i + 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { dst, partner }));
                                        }
                                    }
                                    // perform entanglement swapping
                                    for i in 1..path.len()-1 {
                                        let arg1 = ent_ids[2*i-1].clone();
                                        let arg2 = ent_ids[2*i].clone();
                                        res[path[i]].push(inquir::Expr::from(inquir::EntSwapExpr { arg1, arg2 }));
                                    }
                                    res[from].push(inquir::Expr::from(inquir::QSendExpr { arg: id.clone(), ent: ent_ids[0].clone() }));
                                    res[to].push(inquir::Expr::from(inquir::QRecvExpr { dst: id.clone(), ent: ent_ids[ent_ids.len()-1].clone() }));
                                }
                                let pos1 = allocator.current_pos(&args[0]);
                                let pos2 = allocator.current_pos(&args[1]);
                                assert!(pos1 == pos2);
                                res[pos1 as usize].push(inquir::Expr::from(inquir::ApplyExpr { gate: inquir::PrimitiveGate::CX, args: args.clone() }));
                            }
                        }
                    },
                    hir::PrimitiveGate::X
                    | hir::PrimitiveGate::Y
                    | hir::PrimitiveGate::Z
                    | hir::PrimitiveGate::H
                    | hir::PrimitiveGate::T
                    | hir::PrimitiveGate::Tdg
                    | hir::PrimitiveGate::S => {
                        let pos = allocator.current_pos(&args[0]);
                        res[pos as usize].push(inquir::Expr::from(inquir::ApplyExpr { gate: gate.into(), args }));
                    },
                }
            },
            hir::Expr::Measure(e) => {
                assert!(e.args.len() == 1); // TODO
                let pos = allocator.current_pos(&e.args[0]);
                // TODO: add measure kind
                res[pos as usize].push(inquir::Expr::from(inquir::MeasureExpr { dst: e.dst, args: e.args }));
            },
        }
    }
        res
}
