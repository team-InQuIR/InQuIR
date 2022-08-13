use crate::ast::Expr;

pub struct Metrics {
    comm_depth: u32,
}

impl Metrics {
    pub fn new(es: &Vec<Vec<Expr>>) -> Self {
        Self {
            comm_depth: calc_comm_depth(es),
        }
    }

    pub fn comm_depth(&self) -> u32 {
        self.comm_depth
    }
}

pub fn calc_comm_depth(es: &Vec<Vec<Expr>>) -> u32 {
    0
    //let es = es.clone();
    //let es: Vec<Vec<Expr>> = es.into_iter().map(|es| es.into_iter().filter(|e| match e {
    //    Expr::QSendMove { id: _, ent: _ }
    //    | Expr::QRecv { id: _, ent: _ }
    //    | Expr::RCXC { id: _, ent: _ }
    //    | Expr::RCXT { id: _, ent: _ } => true,
    //    _ => false,
    //}).collect()).collect();

    //let mut depth = 0;
    //let mut iter = vec![0; es.len()];
    //loop {
    //    if iter.iter().enumerate().all(|(i, j)| *j == es[i].len()) {
    //        break;
    //    }
    //    let mut curr_que = HashMap::new();
    //    for i in 0..iter.len() {
    //        if es[i].len() <= iter[i] {
    //            continue;
    //        }
    //        match &es[i][iter[i]] {
    //            Expr::QSendMove { id: _, ent } => {
    //                let node1 = u32::min(i as u32, *node);
    //                let node2 = u32::max(i as u32, *node);
    //                if curr_que.contains_key(&(node1, node2)) {
    //                    *curr_que.get_mut(&(node1, node2)).unwrap() += 1;
    //                } else {
    //                    curr_que.insert((node1, node2), 1);
    //                }
    //            },
    //            Expr::RCXC { id: _, node } => {
    //                let node1 = u32::min(i as u32, *node);
    //                let node2 = u32::max(i as u32, *node);
    //                if curr_que.contains_key(&(node1, node2)) {
    //                    *curr_que.get_mut(&(node1, node2)).unwrap() += 2;
    //                } else {
    //                    curr_que.insert((node1, node2), 2);
    //                }
    //            },
    //            Expr::QRecv { id: _, src: node } => {
    //                let node1 = u32::min(i as u32, *node);
    //                let node2 = u32::max(i as u32, *node);
    //                if curr_que.contains_key(&(node1, node2)) {
    //                    *curr_que.get_mut(&(node1, node2)).unwrap() -= 1;
    //                } else {
    //                    curr_que.insert((node1, node2), -1);
    //                }
    //            },
    //            Expr::RCXT { id: _, node } => {
    //                let node1 = u32::min(i as u32, *node);
    //                let node2 = u32::max(i as u32, *node);
    //                if curr_que.contains_key(&(node1, node2)) {
    //                    *curr_que.get_mut(&(node1, node2)).unwrap() -= 2;
    //                } else {
    //                    curr_que.insert((node1, node2), -2);
    //                }
    //            },
    //            _ => std::unreachable!(),
    //        }
    //    }
    //    for ((node1, node2), count) in curr_que {
    //        if count == 0 { // can establish communication
    //            iter[node1 as usize] += 1;
    //            iter[node2 as usize] += 1;
    //        }
    //    }
    //    depth += 1;
    //}
    //depth
}

pub fn execution_time(es: &Vec<Vec<Expr>>) {
    unimplemented!()
}
