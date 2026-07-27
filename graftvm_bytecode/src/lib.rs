use graftvm_liternal::Liternal;

#[derive(Clone, Debug)]
pub struct Addr {
    pub window: usize,
    pub slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Width {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl Width {
    pub fn is_int(self) -> bool {
        matches!(self, Width::I8 | Width::I16 | Width::I32 | Width::I64)
    }
    pub fn is_uint(self) -> bool {
        matches!(self, Width::U8 | Width::U16 | Width::U32 | Width::U64)
    }
    pub fn is_float(self) -> bool {
        matches!(self, Width::F32 | Width::F64)
    }
}

#[derive(Clone, Debug)]
pub enum Opcode {
    // ── Arithmetic ──
    Add { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Sub { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Mul { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Div { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Rem { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Neg { dst: Addr, src: Addr, ty: Width },

    // ── Compare ──
    Lt { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Le { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Gt { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Ge { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Eq { lhs: Addr, rhs: Addr },
    Neq { lhs: Addr, rhs: Addr },

    // ── Bitwise ──
    And { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Or { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Xor { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Not { dst: Addr, src: Addr, ty: Width },

    // ── Shift ──
    Shl { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },
    Shr { dst: Addr, lhs: Addr, rhs: Addr, ty: Width },

    // ── Conversion ──
    Extend { dst: Addr, src: Addr, signed: bool },
    Trunc { dst: Addr, src: Addr },
    Reinterpret { dst: Addr, src: Addr },
    Convert { dst: Addr, src: Addr, signed: bool },

    // ── Control ──
    Jump(usize),
    Branch(usize),
    /// Call a function: push return address onto return stack, jump to target.
    Call(usize),
    /// Return from a function: pop return address from return stack, jump back.
    Ret,

    // ── Argument passing (cross-window via arg stack) ──
    /// Push a value from a slot onto the argument stack.
    PushArg { src: Addr },
    /// Pop a value from the argument stack into a slot.
    PopArg { dst: Addr },

    // ── Constants ──
    StoreData { index: usize, data: Liternal },
    LoadData { dst: Addr, index: usize },

    // ── Data movement ──
    Move { dst: Addr, src: Addr },
    Copy { dst: Addr, src: Addr },
    Drop { src: Addr },

    // ── Window ──
    Enter,
    Exit,

    // ── System ──
    /// System call: n=0 → write(src to stdout), result stored in dst.
    Syscall { n: u32, src: Addr, dst: Addr },
}

pub type Bytecode = Vec<Opcode>;
