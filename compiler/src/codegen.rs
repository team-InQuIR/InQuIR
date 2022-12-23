pub mod decomposer;
pub mod routing;

use routing::{RemoteOpRouter, RemoteOp};
use inquir;
use inquir::{SessionId, ParticipantId, Label, Process, OpenProc, System, LocProc};
use graph::{
    graph::NodeIndex,
    algo::dijkstra,
};
use crate::{
    arch::{Configuration, configuration::ConnectionGraph},
    codegen::decomposer::Decomposer,
    optimizer,
    hir,
    utils::fresh_ids::{fresh_var_id, fresh_ent_id, fresh_label_id},
};
use std::collections::HashMap;

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

pub fn codegen(exps: Vec<hir::Expr>, config: &Configuration, allocator: Box<dyn RemoteOpRouter>, quasi: bool) -> inquir::System {
    let mut decomposer = Decomposer::new();
    let s = route_telegates(exps, config, allocator);
    println!("[codegen] finish routing.");
    let s = decomposer.decompose(s);
    println!("[codegen] finish decomposition.");
    let s = if quasi {
        let s = optimizer::standardize(s);
        println!("[codegen] finish standardization.");
        s
    } else {
        s
    };
    //let s = optimizer::vectorize(s, config);
    //println!("[codegen] finish vectorization.");
    s
}

fn insert_entswap_chain(program: &mut Vec<Vec<inquir::Process>>, path: Vec<usize>) -> (String, String) {
    let world = SessionId::new("world".to_string()); // TODO
    // entanglement generations
    let ent_ids: Vec<_> = (0..path.len()*2-2).map(|_| format!("_cq{}", fresh_ent_id())).collect();
    let gen_ent_labels: Vec<_> = (0..path.len()-1).map(|_| Label::new(format!("l{}", fresh_label_id()))).collect();
    for i in 0..path.len() {
        if i >= 1 { // path[i] -> path[i - 1]
            let gen = inquir::Process::GenEnt(inquir::GenEntProc {
                x: ent_ids[2*i - 1].clone(),
                p: ParticipantId::new(path[i - 1] as u32),
                label: gen_ent_labels[i - 1].clone(),
            });
            program[path[i]].push(gen);
        }
        if i + 1 < path.len() { // path[i] -> path[i + 1]
            let gen = inquir::Process::GenEnt(inquir::GenEntProc {
                x: ent_ids[2*i].clone(),
                p: ParticipantId::new(path[i + 1] as u32),
                label: gen_ent_labels[i].clone(),
            });
            program[path[i]].push(gen);
        }
    }

    // entanglement swapping chain
    // sending directions: X => i -> i + 1,   Z => i + 1 -> i
    let entswap_labels: Vec<_> = (0..(path.len()-2)*2).map(|_| Label::new(format!("l{}", fresh_label_id()))).collect();
    for i in 1..path.len() {
        if i != 1 {
            let x = format!("_m{}", fresh_var_id());
            let recv = inquir::Process::Recv(inquir::RecvProc {
                s: world.clone(),
                data: (entswap_labels[2*(i-1)-1].clone(), x.clone()),
            });
            let app_x = inquir::Process::Apply(inquir::ApplyProc {
                gate: inquir::PrimitiveGate::X,
                args: vec![ent_ids[2*i-1].clone()],
                ctrl: Some(inquir::Expr::Var(x)),
            });
            program[path[i]].push(recv);
            program[path[i]].push(app_x);
        }
        if i + 1 < path.len() {
            let x1 = format!("_m{}", fresh_var_id());
            let x2 = format!("_m{}", fresh_var_id());
            let entswap = inquir::Process::EntSwap(inquir::EntSwapProc {
                x1: x1.clone(),
                x2: x2.clone(),
                arg1: ent_ids[2*i-1].clone(),
                arg2: ent_ids[2*i].clone(),
            });
            let send_z = inquir::Process::Send(inquir::SendProc {
                s: world.clone(),
                dst: ParticipantId::new(path[0] as u32),
                data: (entswap_labels[2*(i-1)].clone(), inquir::Expr::Var(x1)),
            });
            let send_x = inquir::Process::Send(inquir::SendProc {
                s: world.clone(),
                dst: ParticipantId::new(path[i + 1] as u32),
                data: (entswap_labels[2*(i-1)+1].clone(), inquir::Expr::Var(x2)),
            });
            program[path[i]].push(entswap);
            program[path[i]].push(send_z);
            program[path[i]].push(send_x);

            // endpoint1
            let x = format!("_m{}", fresh_var_id());
            let recv = inquir::Process::Recv(inquir::RecvProc {
                s: world.clone(),
                data: (entswap_labels[2*(i-1)].clone(), x.clone()),
            });
            let app_z = inquir::Process::Apply(inquir::ApplyProc {
                gate: inquir::PrimitiveGate::Z,
                args: vec![ent_ids[0].clone()],
                ctrl: Some(inquir::Expr::Var(x)),
            });
            program[path[0]].push(recv);
            program[path[0]].push(app_z);
        }
    }

    (ent_ids[0].clone(), ent_ids[ent_ids.len() - 1].clone())
}

fn route_telegates(
    exps: Vec<hir::Expr>,
    config: &Configuration,
    mut router: Box<dyn RemoteOpRouter>
) -> inquir::System {
    let mut tele_uid = 0;
    let mut fresh_tele_uid = || {
        let res = tele_uid;
        tele_uid += 1;
        res
    };
    let mut rename = HashMap::new();

    let world_session = SessionId::new("world".to_string());
    let mut res = {
        // First, open a all-to-all session.
        let ps = (0..config.node_size()).map(|i| ParticipantId::new(i as u32)).collect();
        vec![vec![Process::Open(OpenProc { id: world_session.clone(), ps })]; config.node_size()]
    };
    let prevs = build_all_pair_shortest_path(config.connections());
    for e in exps {
        match e {
            hir::Expr::Ret => {},
            hir::Expr::Init(e) => {
                let pos = router.current_pos(&e.dst);
                rename.insert(e.dst.clone(), e.dst.clone());
                res[pos as usize].push(inquir::Process::Init(inquir::InitProc { dst: e.dst }));
            },
            hir::Expr::Apply(hir::ApplyExpr{ gate, args }) => {
                match gate {
                    hir::PrimitiveGate::CX => {
                        match router.next(&args[0], &args[1]) {
                            RemoteOp::LocalCX => {
                                let pos1 = router.current_pos(&args[0]) as usize;
                                let pos2 = router.current_pos(&args[1]) as usize;
                                assert!(pos1 == pos2);
                                res[pos1].push(
                                    inquir::Process::Apply(inquir::ApplyProc {
                                        gate: inquir::PrimitiveGate::CX,
                                        args: args.into_iter().map(|var| rename[&var].clone()).collect(),
                                        ctrl: None
                                    })
                                );
                            },
                            RemoteOp::RCX => {
                                let pos1 = router.current_pos(&args[0]) as usize;
                                let pos2 = router.current_pos(&args[1]) as usize;
                                assert!(pos1 != pos2);
                                let path = construct_shortest_path(&prevs[pos1], pos1, pos2);
                                let (ent1, ent2) = insert_entswap_chain(&mut res, path);
                                let uid = fresh_tele_uid();
                                let label = Label::new(format!("l{}", fresh_label_id()));
                                res[pos1].push(inquir::Process::RCXC(inquir::RCXCProc {
                                    s: world_session.clone(),
                                    p: ParticipantId::new(pos2 as u32),
                                    label: label.clone(),
                                    arg: rename[&args[0]].clone(),
                                    ent: ent1,
                                    uid,
                                }));
                                res[pos2].push(inquir::Process::RCXT(inquir::RCXTProc {
                                    s: world_session.clone(),
                                    p: ParticipantId::new(pos1 as u32),
                                    label,
                                    arg: rename[&args[1]].clone(),
                                    ent: ent2,
                                    uid,
                                }));
                            },
                            RemoteOp::RSwap => {
                                let pos2 = router.current_pos(&args[0]) as usize;
                                let pos1 = router.current_pos(&args[1]) as usize;
                                assert!(pos1 != pos2);
                                let positions = [pos1, pos2];
                                for i in 0..3 { // RSWAP = RCX*3
                                    let pos1 = positions[i % 2];
                                    let pos2 = positions[(i + 1) % 2];
                                    let path = construct_shortest_path(&prevs[pos1], pos1, pos2);
                                    let (ent1, ent2) = insert_entswap_chain(&mut res, path);
                                    let uid = fresh_tele_uid();
                                    let label = Label::new(format!("l{}", fresh_label_id()));
                                    res[pos1].push(inquir::Process::RCXC(inquir::RCXCProc {
                                        s: world_session.clone(),
                                        p: ParticipantId::new(pos2 as u32),
                                        label: label.clone(),
                                        arg: rename[&args[i % 2]].clone(),
                                        ent: ent1,
                                        uid,
                                    }));
                                    res[pos2].push(inquir::Process::RCXT(inquir::RCXTProc {
                                        s: world_session.clone(),
                                        p: ParticipantId::new(pos1 as u32),
                                        label,
                                        arg: rename[&args[(i + 1) % 2]].clone(),
                                        ent: ent2,
                                        uid,
                                    }));
                                }
                                let tmp = rename[&args[0]].clone();
                                *rename.get_mut(&args[0]).unwrap() = rename[&args[1]].clone();
                                *rename.get_mut(&args[1]).unwrap() = tmp;
                            },
                            RemoteOp::Move(id, from, to) => {
                                let from = from as usize;
                                let to = to as usize;
                                let path = construct_shortest_path(&prevs[from], from, to);
                                let (ent1, ent2) = insert_entswap_chain(&mut res, path);
                                let tele_uid = fresh_tele_uid();
                                let label = Label::new(format!("l{}", fresh_label_id()));
                                res[from].push(inquir::Process::QSend(inquir::QSendProc {
                                    s: world_session.clone(),
                                    p: ParticipantId::new(to as u32),
                                    label: label.clone(),
                                    arg: rename[&id].clone(),
                                    ent: ent1,
                                    uid: tele_uid,
                                }));
                                let new = format!("_x{}", fresh_var_id());
                                res[to].push(inquir::Process::QRecv(inquir::QRecvProc {
                                    s: world_session.clone(),
                                    label,
                                    dst: new.clone(),
                                    ent: ent2,
                                    uid: tele_uid,
                                }));
                                *rename.get_mut(&id).unwrap() = new;
                                let pos1 = router.current_pos(&args[0]);
                                let pos2 = router.current_pos(&args[1]);
                                assert!(pos1 == pos2);
                                res[pos1 as usize].push(inquir::Process::Apply(inquir::ApplyProc {
                                    gate: inquir::PrimitiveGate::CX,
                                    args: args.into_iter().map(|var| rename[&var].clone()).collect(),
                                    ctrl: None
                                }));
                            }
                        }
                    },
                    gate => {
                        let pos = router.current_pos(&args[0]) as usize;
                        res[pos].push(inquir::Process::Apply(inquir::ApplyProc {
                            gate: gate.into(),
                            args: args.into_iter().map(|var| rename[&var].clone()).collect(),
                            ctrl: None
                        }));
                    },
                }
            },
            hir::Expr::Measure(e) => {
                assert!(e.args.len() == 1); // TODO
                let pos = router.current_pos(&e.args[0]);
                res[pos as usize].push(inquir::Process::Measure(inquir::MeasureProc {
                    dst: e.dst,
                    args: e.args.into_iter().map(|var| rename[&var].clone()).collect(),
                }));
            },
            hir::Expr::Barrier(_) => {
                // TODO: Currently, this compiler ignore all barriers.
            },
        }
    }

    System::Composition(
        res.into_iter().enumerate()
           .map(|(i, procs)| System::Located(LocProc { p: ParticipantId::new(i as u32), procs })).collect()
    )
}
