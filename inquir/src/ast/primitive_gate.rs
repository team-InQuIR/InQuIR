use std::fmt;

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
