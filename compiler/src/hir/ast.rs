#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ret,

    /// `x = init();`
    Init(InitExpr),

    /// `U(x1, .., xn);`
    Apply(ApplyExpr),

    /// `x = meas(x1, .., xn);`
    Measure(MeasureExpr),
}


impl From<InitExpr> for Expr {
    fn from(e: InitExpr) -> Self {
        Expr::Init(e)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitExpr {
    pub dst: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyExpr {
    pub gate: PrimitiveGate,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureExpr {
    pub kind: MeasureKind,
    pub dst: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveGate {
    X,
    Y,
    Z,
    H,
    T,
    Tdg,
    S,
    CX,
    Rz(f64),
}

impl From<PrimitiveGate> for inquir::PrimitiveGate {
    fn from(kind: PrimitiveGate) -> inquir::PrimitiveGate {
        match kind {
            PrimitiveGate::X => inquir::PrimitiveGate::X,
            PrimitiveGate::Y => inquir::PrimitiveGate::Y,
            PrimitiveGate::Z => inquir::PrimitiveGate::Z,
            PrimitiveGate::H => inquir::PrimitiveGate::H,
            PrimitiveGate::T => inquir::PrimitiveGate::T,
            PrimitiveGate::Tdg => inquir::PrimitiveGate::Tdg,
            PrimitiveGate::CX => inquir::PrimitiveGate::CX,
            PrimitiveGate::Rz(theta) => inquir::PrimitiveGate::Rz(theta),
            PrimitiveGate::S => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasureKind {
    X,
    Z,
}
