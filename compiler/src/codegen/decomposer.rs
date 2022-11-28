use inquir::{
    Expr,
    InitExpr, FreeExpr, ApplyExpr, MeasureExpr, RCXCExpr, RCXTExpr, QSendExpr, QRecvExpr, SendExpr, RecvExpr,
    System, LocExpr,
    PrimitiveGate,
    BExpr,
};
use std::collections::HashMap;

pub struct Decomposer {
    fresh_var_id: u32,
    fresh_ch_id: u32,
    tele_uid_to_ch: HashMap<u32, (String, String)>,
}

impl Decomposer {
    pub fn new() -> Self {
        Self {
            fresh_var_id: 0,
            fresh_ch_id: 0,
            tele_uid_to_ch: HashMap::new(),
        }
    }

    pub fn decompose(&mut self, s: System) -> System {
        match s {
            System::Located(LocExpr { p, exps }) => {
                let exps: Vec<_> = exps.into_iter().map(|e| self.decompose_exp(e)).flatten().collect();
                System::Located(LocExpr { p, exps })
            },
            System::Composition(ss) => {
                System::Composition(ss.into_iter().map(|s| self.decompose(s)).collect())
            },
        }
    }

    fn decompose_exp(&mut self, e: Expr) -> Vec<Expr> {
        match e {
            Expr::RCXC(RCXCExpr { arg, ent, uid }) => {
                let mut res = Vec::new();
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::CX, args: vec![arg.clone(), ent.clone()], ctrl: None }));
                let meas_var = self.fresh_var_id();
                res.push(Expr::Measure(MeasureExpr { dst: meas_var.clone(), args: vec![ent.clone()] }));
                res.push(Expr::Free(FreeExpr { arg: ent }));
                let (ch_name1, ch_name2) = self.get_ch_names(uid, false);
                res.push(Expr::Send(SendExpr { ch: ch_name1, data: meas_var }));
                let recv_var = self.fresh_var_id();
                res.push(Expr::Recv(RecvExpr { ch: ch_name2, data: recv_var.clone() }));
                let ctrl = BExpr::Not(Box::new(BExpr::Var(recv_var)));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::Z, args: vec![arg], ctrl: Some(ctrl) }));
                res
            },
            Expr::RCXT(RCXTExpr { arg, ent, uid }) => {
                let mut res = Vec::new();
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::CX, args: vec![ent.clone(), arg.clone()], ctrl: None }));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::H, args: vec![ent.clone()], ctrl: None }));
                let meas_var = self.fresh_var_id();
                res.push(Expr::Measure(MeasureExpr { dst: meas_var.clone(), args: vec![ent.clone()] }));
                res.push(Expr::Free(FreeExpr { arg: ent }));
                let (ch_name1, ch_name2) = self.get_ch_names(uid, false);
                res.push(Expr::Send(SendExpr { ch: ch_name1, data: meas_var }));
                let recv_var = self.fresh_var_id();
                res.push(Expr::Recv(RecvExpr { ch: ch_name2, data: recv_var.clone() }));
                let ctrl = BExpr::Var(recv_var);
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::X, args: vec![arg], ctrl: Some(ctrl) }));
                res
            },
            Expr::QSend(QSendExpr { arg, ent, uid }) => {
                let mut res = Vec::new();
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::CX, args: vec![arg.clone(), ent.clone()], ctrl: None }));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::H, args: vec![arg.clone()], ctrl: None }));
                let x1 = self.fresh_var_id();
                res.push(Expr::Measure(MeasureExpr { dst: x1.clone(), args: vec![arg.clone()] }));
                let x2 = self.fresh_var_id();
                res.push(Expr::Measure(MeasureExpr { dst: x2.clone(), args: vec![ent.clone()] }));
                res.push(Expr::Free(FreeExpr { arg: ent }));
                let (ch_name1, ch_name2) = self.get_ch_names(uid, true);
                res.push(Expr::Send(SendExpr { ch: ch_name1, data: x1 }));
                res.push(Expr::Send(SendExpr { ch: ch_name2, data: x2 }));
                res
            },
            Expr::QRecv(QRecvExpr { dst, ent, uid }) => {
                let mut res = Vec::new();
                res.push(Expr::Init(InitExpr { dst: dst.clone() }));
                let (ch_name1, ch_name2) = self.get_ch_names(uid, true);
                let x1 = self.fresh_var_id();
                let x2 = self.fresh_var_id();
                res.push(Expr::Recv(RecvExpr { ch: ch_name1, data: x1.clone() }));
                res.push(Expr::Recv(RecvExpr { ch: ch_name2, data: x2.clone() }));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::Z, args: vec![ent.clone()], ctrl: Some(BExpr::Var(x1)) }));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::X, args: vec![ent.clone()], ctrl: Some(BExpr::Var(x2)) }));
                // swap
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::CX, args: vec![ent.clone(), dst.clone()], ctrl: None }));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::CX, args: vec![dst.clone(), ent.clone()], ctrl: None }));
                res.push(Expr::Apply(ApplyExpr { gate: PrimitiveGate::CX, args: vec![ent.clone(), dst], ctrl: None }));
                res.push(Expr::Free(FreeExpr { arg: ent }));
                res
            },
            Expr::Parallel(_) => unimplemented!(),
            e => vec![e],
        }
    }

    fn fresh_var_id(&mut self) -> String {
        let id = self.fresh_var_id;
        self.fresh_var_id += 1;
        format!("_m{}", id)
    }

    fn get_ch_names(&mut self, uid: u32, is_teledata: bool) -> (String, String) {
        if self.tele_uid_to_ch.contains_key(&uid) {
            self.tele_uid_to_ch[&uid].clone()
        } else {
            let name1 = format!("_a{}", self.fresh_ch_id);
            let name2 = format!("_a{}", self.fresh_ch_id + 1);
            self.tele_uid_to_ch.insert(uid, (name1.clone(), name2.clone()));
            self.fresh_ch_id += 2;
            if is_teledata {
                (name1, name2)
            } else {
                (name2, name1)
            }
        }
    }
}
