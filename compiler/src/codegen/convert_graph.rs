use crate::{
    arch::Configuration,
    dependency_graph::DependencyGraph,
};
use inquir::{
    ParticipantId,
    Process,
    System, LocProc,
};
use std::collections::{VecDeque, HashMap};

pub fn convert_graph(g: DependencyGraph, config: &Configuration) -> System {
    let mut in_deg = vec![0; g.node_count()];
    let mut que = VecDeque::new();
    for i in 0..g.node_count() {
        in_deg[i] = g.node(i).incoming().len();
        if in_deg[i] == 0 {
            que.push_back(i);
        }
    }
    let mut cap = vec![vec![0; config.node_size()]; config.node_size()];
    config.connections().edges().iter().for_each(|e| {
        let s = e.source();
        let t = e.target();
        cap[s][t] += 1;
        cap[t][s] += 1;
    });
    let mut partner = HashMap::new();
    let mut res = vec![Vec::new(); config.node_size()];
    let mut ent_que = vec![vec![VecDeque::new(); config.node_size()]; config.node_size()];
    while let Some(idx) = que.pop_front() {
        let (p, process) = g.node(idx).weight();
        match process {
            Process::GenEnt(proc) => {
                let from = p.to_usize();
                let to = proc.p.to_usize();
                if cap[from][to] > 0 {
                    cap[from][to] -= 1;
                    res[from].push(Process::GenEnt(proc.clone()));
                    partner.insert(proc.x.clone(), to);
                } else {
                    ent_que[from][to].push_back((idx, proc.clone()));
                    continue;
                }
            },
            Process::EntSwap(proc) => {
                let from = p.to_usize();
                res[from].push(Process::EntSwap(proc.clone()));
                [&proc.arg1, &proc.arg2].into_iter().for_each(|var| {
                    let to = partner[var];
                    partner.remove(var);
                    if let Some((idx2, proc)) = ent_que[from][to].pop_front() {
                        partner.insert(proc.x.clone(), to);
                        res[from].push(Process::GenEnt(proc));
                        g.outgoing_edges(idx2).iter().for_each(|&eidx| {
                            let to = g.edge(eidx).target();
                            in_deg[to] -= 1;
                            if in_deg[to] == 0 {
                                que.push_back(to);
                            }
                        });
                    }
                });
            },
            Process::Free(proc) => {
                let from = p.to_usize();
                let to = partner[&proc.arg.clone()];
                res[from].push(Process::Free(proc.clone()));
                partner.remove(&proc.arg.clone()).expect("failed to free");
                if let Some((idx2, proc)) = ent_que[from][to].pop_front() {
                    partner.insert(proc.x.clone(), to);
                    res[from].push(Process::GenEnt(proc));
                    g.outgoing_edges(idx2).iter().for_each(|&eidx| {
                        let to = g.edge(eidx).target();
                        in_deg[to] -= 1;
                        if in_deg[to] == 0 {
                            que.push_back(to);
                        }
                    });
                } else {
                    cap[from][to] += 1;
                }
            },
            // These operations must be decomposed.
            Process::QSend(_) => todo!(),
            Process::QRecv(_) => todo!(),
            Process::RCXC(_) => todo!(),
            Process::RCXT(_) => todo!(),
            Process::Parallel(_) => unimplemented!(),
            process => res[p.to_usize()].push(process.clone()),
        }
        g.outgoing_edges(idx).iter().for_each(|&eidx| {
            let to = g.edge(eidx).target();
            in_deg[to] -= 1;
            if in_deg[to] == 0 {
                que.push_back(to);
            }
        });
    }
    for i in 0..config.node_size() {
        for j in 0..config.node_size() {
            assert!(ent_que[i][j].is_empty());
        }
    }

    System::Composition(
        res.into_iter().enumerate().map(|(p, procs)| {
            System::Located(LocProc { p: ParticipantId::new(p as u32), procs })
        }).collect()
    )
}
