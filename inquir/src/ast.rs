use crate::bexp::BExpr;
use std::convert::From;
use std::fmt;

pub type ProcessorId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `x = init();`
    Init(InitExpr),

    /// `free x;`
    Free(FreeExpr),

    /// Generate an entanglement with the `n`th node.
    /// `x = genEnt p;`
    GenEnt(GenEntExpr),

    /// Entangle swapping
    /// `entSwap x1 x2;`
    EntSwap(EntSwapExpr),

    /// `qsend x via y;`.
    QSend(QSendExpr),

    /// `x = qrecv via y;`
    QRecv(QRecvExpr),

    /// A classical sending instruction: `x!<y>;`.
    Send(SendExpr),

    /// A classical receiving instruction: `x!<y>;`.
    Recv(RecvExpr),

    /// Remote CX gate (controlled side)
    /// `rcxc x via y;`
    RCXC(RCXCExpr),

    /// Remote CX gate (target side)
    /// `rcxt x via y;`
    RCXT(RCXTExpr),

    /// `U(x1, .., xn);`
    Apply(ApplyExpr),

    /// `x = measure(y1, .., yn)`
    Measure(MeasureExpr),

    /// Execute local instructions parallely
    Parallel(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum System {
    /// a located expression `[e]p`.
    Located(LocExpr),

    /// a composition of systems: `P1 | P2`.
    Composition(Vec<System>),
}


impl From<InitExpr> for Expr {
    fn from(e: InitExpr) -> Self {
        Expr::Init(e)
    }
}

impl From<FreeExpr> for Expr {
    fn from(e: FreeExpr) -> Self {
        Expr::Free(e)
    }
}

impl From<GenEntExpr> for Expr {
    fn from(e: GenEntExpr) -> Self {
        Expr::GenEnt(e)
    }
}

impl From<EntSwapExpr> for Expr {
    fn from(e: EntSwapExpr) -> Self {
        Expr::EntSwap(e)
    }
}

impl From<QSendExpr> for Expr {
    fn from(e: QSendExpr) -> Self {
        Expr::QSend(e)
    }
}

impl From<QRecvExpr> for Expr {
    fn from(e: QRecvExpr) -> Self {
        Expr::QRecv(e)
    }
}

impl From<ApplyExpr> for Expr {
    fn from(e: ApplyExpr) -> Self {
        Expr::Apply(e)
    }
}

impl From<MeasureExpr> for Expr {
    fn from(e: MeasureExpr) -> Self {
        Expr::Measure(e)
    }
}

impl From<RCXCExpr> for Expr {
    fn from(e: RCXCExpr) -> Self {
        Expr::RCXC(e)
    }
}

impl From<RCXTExpr> for Expr {
    fn from(e: RCXTExpr) -> Self {
        Expr::RCXT(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitExpr {
    pub dst: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeExpr {
    pub arg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenEntExpr {
    pub label: String,
    pub partner: ProcessorId,
    pub uid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntSwapExpr {
    pub arg1: String,
    pub arg2: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QSendExpr {
    pub arg: String,
    pub ent: String,
    pub uid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QRecvExpr {
    pub dst: String,
    pub ent: String,
    pub uid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendExpr {
    pub ch: String,
    pub data: BExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecvExpr {
    pub ch: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RCXCExpr {
    pub arg: String,
    pub ent: String,
    pub uid: u32, // annotation for compilers
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RCXTExpr {
    pub arg: String,
    pub ent: String,
    pub uid: u32, // annocation for compilers
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyExpr {
    pub gate: PrimitiveGate,
    pub args: Vec<String>,
    pub ctrl: Option<BExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureExpr {
    pub dst: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocExpr {
    pub p: ProcessorId,
    pub exps: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveGate {
    I,
    X,
    Y,
    Z,
    H,
    T,
    Tdg,
    S,
    CX,
    RCX, // Remote CX
    Rz(f64), // Rotate Z. (TODO: this gate will be removed in future.)
}

impl Expr {
    pub fn is_app(&self) -> bool {
        match self {
            Expr::Apply(_) => true,
            _ => false,
        }
    }

    pub fn is_measure(&self) -> bool {
        match self {
            Expr::Measure(_) => true,
            _ => false
        }
    }

    pub fn as_app(&self) -> Option<ApplyExpr> {
        match self {
            Expr::Apply(app) => Some(app.clone()),
            _ => None,
        }
    }

    pub fn as_app_mut(&mut self) -> Option<&mut ApplyExpr> {
        match self {
            Expr::Apply(app) => Some(app),
            _ => None,
        }
    }

    pub fn as_measure(&self) -> Option<MeasureExpr> {
        match self {
            Expr::Measure(e) => Some(e.clone()),
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::GenEnt(GenEntExpr { label, partner, uid: _ }) => write!(f, "{} = genEnt via {}", label, partner),
            Expr::EntSwap(EntSwapExpr { arg1, arg2 }) => write!(f, "entSwap {} {}", arg1, arg2),
            Expr::Init(InitExpr { dst }) => write!(f, "{} = init()", dst),
            Expr::Free(FreeExpr { arg }) => write!(f, "free {}", arg),
            Expr::QSend(QSendExpr { arg, ent, uid: _ }) => write!(f, "qsend {} via {}", arg, ent),
            Expr::QRecv(QRecvExpr { dst, ent, uid: _ }) => write!(f, "{} = qrecv via {}", dst, ent),
            Expr::Send(SendExpr { ch, data }) => write!(f, "send {} via {}", data, ch),
            Expr::Recv(RecvExpr { ch, data }) => write!(f, "{} = recv via {}", data, ch),
            Expr::RCXC(RCXCExpr { arg, ent, uid: _ }) => write!(f, "rcxc {} via {}", arg, ent),
            Expr::RCXT(RCXTExpr { arg, ent, uid: _ }) => write!(f, "rcxt {} via {}", arg, ent),
            Expr::Apply(ApplyExpr { gate, args, ctrl }) => {
                let args_str: Vec<_> = args.iter().map(|arg| arg.clone()).collect();
                let args_str = args_str.join(" ");
                if let Some(b) = ctrl {
                    write!(f, "{} {} ctrl {}", gate, args_str, b)
                } else {
                    write!(f, "{} {}", gate, args_str)
                }
            },
            Expr::Measure(MeasureExpr { dst, args }) => {
                let args_str: Vec<_> = args.iter().map(|arg| arg.clone()).collect();
                let args_str = args_str.join(" ");
                write!(f, "{} = measure {}", dst, args_str)
            },
            Expr::Parallel(es) => {
                let s: Vec<String> = es.iter().map(|e| format!("{}", e)).collect();
                let s = s.join(" | ");
                write!(f, "{}", s)
            },
        }
    }
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            System::Located(LocExpr { p, exps }) => {
                let mut s = String::new();
                s += format!("{} {{\n", p).as_str();
                exps.iter().for_each(|e| {
                    s += format!("  {};\n", e).as_str();
                });
                s += "\n}";
                write!(f, "{}", s)
            },
            System::Composition(ss) => {
                let s: Vec<_> = ss.iter().map(|s| format!("{}", s)).collect();
                let s = s.join("\n");
                write!(f, "{}", s)
            },
        }
    }
}

impl fmt::Display for PrimitiveGate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrimitiveGate::I => write!(f, "I"),
            PrimitiveGate::X => write!(f, "X"),
            PrimitiveGate::Y => write!(f, "Y"),
            PrimitiveGate::Z => write!(f, "Z"),
            PrimitiveGate::H => write!(f, "H"),
            PrimitiveGate::T => write!(f, "T"),
            PrimitiveGate::Tdg => write!(f, "Tdg"),
            PrimitiveGate::S => write!(f, "S"),
            PrimitiveGate::CX => write!(f, "CX"),
            PrimitiveGate::RCX => write!(f, "RCX"),
            PrimitiveGate::Rz(r) => write!(f, "Rz({})", r),
        }
    }
}
