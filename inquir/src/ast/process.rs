use crate::ast::expr::*;
use crate::ast::PrimitiveGate;
use crate::ast::{
    SessionId,
    Label,
    ParticipantId,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Process {
    Open(OpenProc),

    /// `x = init();`
    Init(InitProc),

    /// `free x;`
    Free(FreeProc),

    /// Generate an entanglement with the `n`th node.
    /// `x = genEnt[p](l);`
    GenEnt(GenEntProc),

    /// Entangle swapping
    /// `(x1, x2) = entSwap(y1, y2);`
    EntSwap(EntSwapProc),

    /// `qsend[p](s, l, x1, x2);`.
    QSend(QSendProc),

    /// `x = qrecv(s, l, x);`
    QRecv(QRecvProc),

    /// A classical sending instruction: `s[p]!<l : e>;`.
    Send(SendProc),

    /// A classical receiving instruction: `s?(l : y);`.
    Recv(RecvProc),

    /// Remote CX gate (controlled side)
    /// `rcxc x via y;`
    RCXC(RCXCProc),

    /// Remote CX gate (target side)
    /// `rcxt x via y;`
    RCXT(RCXTProc),

    /// `U(x1, .., xn);`
    Apply(ApplyProc),

    /// `x = measure(y1, .., yn)`
    Measure(MeasureProc),

    /// Execute local instructions parallely
    Parallel(Vec<Process>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenProc {
    pub id: SessionId,
    pub ps: Vec<ParticipantId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitProc {
    pub dst: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeProc {
    pub arg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenEntProc {
    pub x: String,
    pub p: ParticipantId,
    pub label: Label,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntSwapProc {
    pub x1: String,
    pub x2: String,
    pub arg1: String,
    pub arg2: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QSendProc {
    pub p: ParticipantId,
    pub s: SessionId,
    pub label: Label,
    pub arg: String,
    pub ent: String,
    pub uid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QRecvProc {
    pub s: SessionId,
    pub label: Label,
    pub dst: String,
    pub ent: String,
    pub uid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendProc {
    pub s: SessionId,
    pub dst: ParticipantId, // destination
    pub data: (Label, Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecvProc {
    pub s: SessionId,
    pub data: (Label, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RCXCProc {
    pub s: SessionId,
    pub p: ParticipantId,
    pub label: Label,
    pub arg: String,
    pub ent: String,
    pub uid: u32, // annotation for compilers
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RCXTProc {
    pub s: SessionId,
    pub p: ParticipantId,
    pub label: Label,
    pub arg: String,
    pub ent: String,
    pub uid: u32, // annocation for compilers
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyProc {
    pub gate: PrimitiveGate,
    pub args: Vec<String>,
    pub ctrl: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureProc {
    pub dst: String,
    pub args: Vec<String>,
}

impl Process {
    pub fn is_app(&self) -> bool {
        match self {
            Process::Apply(_) => true,
            _ => false,
        }
    }

    pub fn is_measure(&self) -> bool {
        match self {
            Process::Measure(_) => true,
            _ => false
        }
    }

    pub fn as_app(&self) -> Option<ApplyProc> {
        match self {
            Process::Apply(app) => Some(app.clone()),
            _ => None,
        }
    }

    pub fn as_app_mut(&mut self) -> Option<&mut ApplyProc> {
        match self {
            Process::Apply(app) => Some(app),
            _ => None,
        }
    }

    pub fn as_measure(&self) -> Option<MeasureProc> {
        match self {
            Process::Measure(e) => Some(e.clone()),
            _ => None,
        }
    }
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Process::Open(OpenProc { id, ps }) => {
                write!(f, "{} = open[", id.to_string())?;
                let ss: Vec<_> = ps.iter().map(|&pid| pid.to_u32().to_string()).collect();
                let ss = ss.join(",");
                write!(f, "{}]", ss)
            },
            Process::GenEnt(GenEntProc { x, p, label }) => write!(f, "{} = genEnt[{}]({})", x, p, label),
            Process::EntSwap(EntSwapProc { x1, x2, arg1, arg2 }) => write!(f, "({}, {}) = entSwap({}, {})", x1, x2, arg1, arg2),
            Process::Init(InitProc { dst }) => write!(f, "{} = init()", dst),
            Process::Free(FreeProc { arg }) => write!(f, "free {}", arg),
            Process::QSend(QSendProc { s, p, label, arg, ent, uid: _ })
                => write!(f, "qsend[{}]({}, {}, {}, {})", p, s, label, arg, ent),
            Process::QRecv(QRecvProc { s, label, dst, ent, uid: _ })
                => write!(f, "{} = qrecv({}, {}, {})", dst, s, label, ent),
            Process::Send(SendProc { s, dst, data: (lbl, exp) }) => write!(f, "send[{}]({}, {}:{})", dst, s, lbl, exp),
            Process::Recv(RecvProc { s, data: (lbl, var) }) => write!(f, "recv({}, {}:{})", s, lbl, var),
            Process::RCXC(RCXCProc { s, p, label, arg, ent, uid: _ })
                => write!(f, "rcxc[{}]({}, {}, {}, {})", p, s, label, arg, ent),
            Process::RCXT(RCXTProc { s, p, label, arg, ent, uid: _ })
                => write!(f, "rcxt[{}]({}, {}, {}, {})", p, s, label, arg, ent),
            Process::Apply(ApplyProc { gate, args, ctrl }) => {
                let args_str: Vec<_> = args.iter().map(|arg| arg.clone()).collect();
                let args_str = args_str.join(" ");
                if let Some(b) = ctrl {
                    write!(f, "{}[{}] {}", gate, b, args_str)
                } else {
                    write!(f, "{} {}", gate, args_str)
                }
            },
            Process::Measure(MeasureProc { dst, args }) => {
                let args_str: Vec<_> = args.iter().map(|arg| arg.clone()).collect();
                let args_str = args_str.join(" ");
                write!(f, "{} = measure {}", dst, args_str)
            },
            Process::Parallel(es) => {
                let s: Vec<String> = es.iter().map(|e| format!("{}", e)).collect();
                let s = s.join(" | ");
                write!(f, "{}", s)
            },
        }
    }
}
