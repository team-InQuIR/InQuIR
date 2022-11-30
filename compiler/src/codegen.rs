pub mod allocation;
pub mod always_rcx;
pub mod decomposer;

use allocation::NodeAllocator;
use inquir;
use inquir::{System, LocExpr};
use graph::{
    graph::NodeIndex,
    algo::dijkstra,
};
use crate::{
    arch::{Configuration, configuration::ConnectionGraph},
    codegen::decomposer::Decomposer,
    optimizer,
    hir,
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

    path.into_iter().rev().collect()
}

pub fn codegen(exps: Vec<hir::Expr>, config: &Configuration, allocator: Box<dyn NodeAllocator>) -> inquir::System {
    let mut decomposer = Decomposer::new();
    let s = route_telegates(exps, config, allocator);
    println!("[codegen] finish routing.");
    let s = decomposer.decompose(s);
    println!("[codegen] finish decomposition.");
    let s = optimizer::vectorize(s, config);
    println!("[codegen] finish vectorization.");
    s
}

fn route_telegates(
    exps: Vec<hir::Expr>,
    config: &Configuration,
    mut allocator: Box<dyn NodeAllocator>
) -> inquir::System {
    let mut fresh_ent_id_ = 0;
    let mut fresh_ent_id = || {
        let res = format!("ent{}", fresh_ent_id_);
        fresh_ent_id_ += 1;
        res
    };
    let mut tele_uid = 0;

    let mut res = vec![vec![]; config.node_size()];
    let prevs = build_all_pair_shortest_path(config.connections());
    for e in exps {
        match e {
            hir::Expr::Ret => {},
            hir::Expr::Init(e) => {
                let pos = allocator.current_pos(&e.dst);
                res[pos as usize].push(inquir::Expr::from(inquir::InitExpr { dst: e.dst }));
            },
            hir::Expr::Apply(hir::ApplyExpr{ gate, args }) => {
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
                                            args: args.clone(),
                                            ctrl: None
                                        }));
                                } else {
                                    let pos1 = pos1 as NodeIndex;
                                    let pos2 = pos2 as NodeIndex;
                                    let path = construct_shortest_path(&prevs[pos1], pos1, pos2);
                                    let ent_ids: Vec<_> = (0..path.len()*2-2).map(|_| fresh_ent_id()).collect();
                                    // generate entanglements
                                    for i in 0..path.len() {
                                        if i >= 1 {
                                            let label = ent_ids[2*i-1].clone();
                                            let partner = path[i - 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { label, partner }));
                                        }
                                        if i + 1 < path.len() {
                                            let label = ent_ids[2*i].clone();
                                            let partner = path[i + 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { label, partner }));
                                        }
                                    }
                                    // perform entanglement swapping
                                    for i in 1..path.len()-1 {
                                        let arg1 = ent_ids[2*i-1].clone();
                                        let arg2 = ent_ids[2*i].clone();
                                        res[path[i]].push(inquir::Expr::from(inquir::EntSwapExpr { arg1, arg2 }));
                                    }
                                    res[pos1].push(inquir::Expr::from(inquir::RCXCExpr {
                                        arg: args[0].clone(),
                                        ent: ent_ids[0].clone(),
                                        uid: tele_uid,
                                    }));
                                    res[pos2].push(inquir::Expr::from(inquir::RCXTExpr {
                                        arg: args[1].clone(),
                                        ent: ent_ids[ent_ids.len()-1].clone(),
                                        uid: tele_uid,
                                    }));
                                    tele_uid += 1;
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
                                            let label = ent_ids[2*i-1].clone();
                                            let partner = path[i - 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { label, partner }));
                                        }
                                        if i + 1 < path.len() {
                                            let label = ent_ids[2*i].clone();
                                            let partner = path[i + 1] as inquir::ProcessorId;
                                            res[path[i]].push(inquir::Expr::from(inquir::GenEntExpr { label, partner }));
                                        }
                                    }
                                    // perform entanglement swapping
                                    for i in 1..path.len()-1 {
                                        let arg1 = ent_ids[2*i-1].clone();
                                        let arg2 = ent_ids[2*i].clone();
                                        res[path[i]].push(inquir::Expr::from(inquir::EntSwapExpr { arg1, arg2 }));
                                    }
                                    res[from].push(inquir::Expr::from(inquir::QSendExpr {
                                        arg: id.clone(),
                                        ent: ent_ids[0].clone(),
                                        uid: tele_uid,
                                    }));
                                    res[to].push(inquir::Expr::from(inquir::QRecvExpr {
                                        dst: id.clone(),
                                        ent: ent_ids[ent_ids.len()-1].clone(),
                                        uid: tele_uid,
                                    }));
                                    tele_uid += 1;
                                }
                                let pos1 = allocator.current_pos(&args[0]);
                                let pos2 = allocator.current_pos(&args[1]);
                                assert!(pos1 == pos2);
                                res[pos1 as usize].push(inquir::Expr::from(inquir::ApplyExpr {
                                    gate: inquir::PrimitiveGate::CX,
                                    args: args.clone(),
                                    ctrl: None
                                }));
                            }
                        }
                    },
                    gate => {
                        let pos = allocator.current_pos(&args[0]);
                        res[pos as usize].push(inquir::Expr::from(inquir::ApplyExpr { gate: gate.into(), args, ctrl: None }));
                    },
                }
            },
            hir::Expr::Measure(e) => {
                assert!(e.args.len() == 1); // TODO
                let pos = allocator.current_pos(&e.args[0]);
                // TODO: add measure kind
                res[pos as usize].push(inquir::Expr::from(inquir::MeasureExpr { dst: e.dst, args: e.args }));
            },
            hir::Expr::Barrier(_) => {
                // TODO: Currently, this compiler ignore all barriers.
            },
        }
    }

    System::Composition(
        res.into_iter().enumerate()
           .map(|(i, exps)| System::Located(LocExpr { p: i as u32, exps })).collect()
    )
}
