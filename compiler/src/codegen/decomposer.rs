use inquir::{
    Label,
    Process,
    InitProc, FreeProc, ApplyProc, MeasureProc, RCXCProc, RCXTProc, QSendProc, QRecvProc, SendProc, RecvProc,
    System, LocProc,
    PrimitiveGate,
    Expr,
};
use crate::utils::fresh_ids::fresh_var_id;

pub struct Decomposer {
}

impl Decomposer {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn decompose(&mut self, s: System) -> System {
        match s {
            System::Located(LocProc { p, procs }) => {
                let procs: Vec<_> = procs.into_iter().map(|proc| self.decompose_proc(proc)).flatten().collect();
                System::Located(LocProc { p, procs })
            },
            System::Composition(ss) => {
                System::Composition(ss.into_iter().map(|s| self.decompose(s)).collect())
            },
        }
    }

    fn decompose_proc(&mut self, e: Process) -> Vec<Process> {
        match e {
            Process::RCXC(RCXCProc { s, p, label, arg, ent, uid: _ }) => {
                let mut res = Vec::new();
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::CX, args: vec![arg.clone(), ent.clone()], ctrl: None }));
                let meas_var = self.fresh_var_id();
                res.push(Process::Measure(MeasureProc { dst: meas_var.clone(), args: vec![ent.clone()] }));
                res.push(Process::Free(FreeProc { arg: ent }));
                let label2 = Label::new(label.to_string() + "_2");
                res.push(Process::Send(SendProc { s: s.clone(), dst: p, data: (label, Expr::Var(meas_var)) }));
                let recv_var = self.fresh_var_id();
                res.push(Process::Recv(RecvProc { s, data: (label2, recv_var.clone()) }));
                let ctrl = Expr::Var(recv_var);
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::Z, args: vec![arg], ctrl: Some(ctrl) }));
                res
            },
            Process::RCXT(RCXTProc { s, p, label, arg, ent, uid: _ }) => {
                let mut res = Vec::new();
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::CX, args: vec![ent.clone(), arg.clone()], ctrl: None }));
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::H, args: vec![ent.clone()], ctrl: None }));
                let meas_var = self.fresh_var_id();
                res.push(Process::Measure(MeasureProc { dst: meas_var.clone(), args: vec![ent.clone()] }));
                res.push(Process::Free(FreeProc { arg: ent }));
                let label2 = Label::new(label.to_string() + "_2");
                res.push(Process::Send(SendProc { s: s.clone(), dst: p, data: (label2, Expr::Var(meas_var)) }));
                let recv_var = self.fresh_var_id();
                res.push(Process::Recv(RecvProc { s, data: (label, recv_var.clone()) }));
                let ctrl = Expr::Var(recv_var);
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::X, args: vec![arg], ctrl: Some(ctrl) }));
                res
            },
            Process::QSend(QSendProc { s, p, label, arg, ent, uid: _ }) => {
                let mut res = Vec::new();
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::CX, args: vec![arg.clone(), ent.clone()], ctrl: None }));
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::H, args: vec![arg.clone()], ctrl: None }));
                let x1 = self.fresh_var_id();
                res.push(Process::Measure(MeasureProc { dst: x1.clone(), args: vec![arg.clone()] }));
                let x2 = self.fresh_var_id();
                res.push(Process::Measure(MeasureProc { dst: x2.clone(), args: vec![ent.clone()] }));
                res.push(Process::Free(FreeProc { arg: ent }));
                let label2 = Label::new(label.to_string() + "_2");
                res.push(Process::Send(SendProc { s: s.clone(), dst: p, data: (label, Expr::Var(x1)) }));
                res.push(Process::Send(SendProc { s, dst: p, data: (label2, Expr::Var(x2)) }));
                res
            },
            Process::QRecv(QRecvProc { s, label, dst, ent, uid: _ }) => {
                let mut res = Vec::new();
                res.push(Process::Init(InitProc { dst: dst.clone() }));
                let x1 = self.fresh_var_id();
                let x2 = self.fresh_var_id();
                let label2 = Label::new(label.to_string() + "_2");
                res.push(Process::Recv(RecvProc { s: s.clone(), data: (label, x1.clone()) }));
                res.push(Process::Recv(RecvProc { s, data: (label2, x2.clone()) }));
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::Z, args: vec![ent.clone()], ctrl: Some(Expr::Var(x1)) }));
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::X, args: vec![ent.clone()], ctrl: Some(Expr::Var(x2)) }));
                // swap
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::CX, args: vec![ent.clone(), dst.clone()], ctrl: None }));
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::CX, args: vec![dst.clone(), ent.clone()], ctrl: None }));
                res.push(Process::Apply(ApplyProc { gate: PrimitiveGate::CX, args: vec![ent.clone(), dst], ctrl: None }));
                res.push(Process::Free(FreeProc { arg: ent }));
                res
            },
            Process::Parallel(_) => unimplemented!(),
            e => vec![e],
        }
    }

    fn fresh_var_id(&mut self) -> String {
        format!("_m{}", fresh_var_id())
    }
}
