use inquir::{
    ParticipantId,
    Process, FreeProc, GenEntProc, EntSwapProc, QSendProc, QRecvProc, RCXCProc, RCXTProc,
    System, LocProc
};
use crate::{
    arch::Configuration,
    dependency_graph::DependencyGraphBuilder,
};
use std::collections::{VecDeque, HashMap};
use indicatif::ProgressBar;

pub fn vectorize(s: System, config: &Configuration) -> System {
    let n = config.node_size();
    let builder = DependencyGraphBuilder::new();
    let dep_g = builder.build(s);
    let pb = ProgressBar::new(dep_g.node_count() as u64);

    let mut que = VecDeque::new();
    let mut in_deg: Vec<_> = (0..dep_g.node_count()).map(|i| dep_g.incoming_edges(i).len()).collect();
    in_deg.iter().enumerate().for_each(|(i, &d)| if d == 0 { que.push_back(i); });
    let mut capacity = vec![vec![0; n]; n];
    config.connections().edges().iter().for_each(|e| {
        capacity[e.source()][e.target()] += 1;
        capacity[e.target()][e.source()] += 1;
    });
    let mut current_partners = vec![HashMap::new(); n];
    let mut res = vec![Vec::new(); config.node_size()];
    while que.len() > 0 {
        let mut processed_count = 0;
        let mut nxt_que = VecDeque::new();
        let mut tmp = vec![Vec::new(); n];
        while let Some(i) = que.pop_front() {
            let v = dep_g.node(i);
            let (p, e) = v.weight();
            // check the current capacity to generate a new entanglement
            match e {
                Process::GenEnt(GenEntProc { x, p: another, label: _ }) => {
                    let s = p.to_usize();
                    let t = another.to_usize();
                    if capacity[s][t] == 0 { // postpone
                        nxt_que.push_back(i);
                        continue;
                    } else {
                        capacity[s][t] -= 1;
                        current_partners[s].insert(x, t);
                    }
                },
                Process::Free(FreeProc { arg }) => {
                    let s = p.to_usize();
                    if current_partners[s].contains_key(&arg) { // entanglements
                        let t = current_partners[s][&arg];
                        capacity[s][t] += 1;
                        current_partners[s].remove(arg);
                    }
                },
                Process::Parallel(_) => unimplemented!(),
                _ => {},
            }
            tmp[p.to_usize()].push(e.clone());
            v.outgoing().iter().for_each(|&eidx| {
                let e = dep_g.edge(eidx);
                in_deg[e.target()] -= 1;
                if in_deg[e.target()] == 0 {
                    nxt_que.push_back(e.target());
                }
            });
        }

        tmp.into_iter().enumerate().for_each(|(p, exps)| {
            // obtain capacities from entanglement consumptions
            exps.iter().for_each(|e| {
                match e {
                    Process::EntSwap(EntSwapProc { x1: _, x2: _, arg1, arg2 }) => {
                        let t1 = current_partners[p][arg1];
                        let t2 = current_partners[p][arg2];
                        capacity[p][t1] += 1;
                        capacity[p][t2] += 1;
                        current_partners[p].remove(arg1);
                        current_partners[p].remove(arg2);
                    },
                    Process::QSend(QSendProc { s: _, p: _, label: _, arg: _, ent, uid: _ })
                    | Process::QRecv(QRecvProc { s: _, label: _, dst: _, ent, uid: _ })
                    | Process::RCXC(RCXCProc { s: _, p: _, label: _, arg: _, ent, uid: _ })
                    | Process::RCXT(RCXTProc { s: _, p: _, label: _, arg: _, ent, uid: _ }) => {
                        let t = current_partners[p][ent];
                        capacity[p][t] += 1;
                        current_partners[p].remove(ent);
                    },
                    Process::Parallel(_) => unimplemented!(),
                    _ => {}
                }
            });

            processed_count += exps.len();

            // parallelize
            if exps.len() > 0 {
                let e = if exps.len() > 1 {
                    Process::Parallel(exps)
                } else {
                    exps[0].clone()
                };
                res[p].push(e);
            }
        });

        que = nxt_que;
        pb.inc(processed_count as u64);
    }

    pb.finish_with_message("Done vectorization");

    let located_procs: Vec<_> = res.into_iter().enumerate().filter_map(|(p, procs)| {
        if procs.len() == 0 {
            None
        } else {
            Some(System::Located(LocProc { p: ParticipantId::new(p as u32), procs }))
        }
    }).collect();
    if located_procs.len() > 1 {
        System::Composition(located_procs)
    } else {
        located_procs[0].clone()
    }
}
