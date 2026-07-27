use std::fmt::Debug;
use std::fmt;

pub use crate::liternal::{float::Float, int::Int, uint::UInt};

mod float;
mod int;
mod uint;

#[derive(Clone, PartialEq)]
pub enum Liternal {
    Int(Int),
    UInt(UInt),
    Float(Float),
    String(String),
    Bool(bool),
}

impl Liternal {
    pub fn as_int(&self) -> Option<&Int> {
        match self {
            Liternal::Int(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_uint(&self) -> Option<&UInt> {
        match self {
            Liternal::UInt(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_float(&self) -> Option<&Float> {
        match self {
            Liternal::Float(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Liternal::String(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Liternal::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn expect_int(&self) -> Result<&Int, String> {
        self.as_int()
            .ok_or_else(|| format!("expected Int, got {:?}", self))
    }
    pub fn expect_uint(&self) -> Result<&UInt, String> {
        self.as_uint()
            .ok_or_else(|| format!("expected UInt, got {:?}", self))
    }
    pub fn expect_float(&self) -> Result<&Float, String> {
        self.as_float()
            .ok_or_else(|| format!("expected Float, got {:?}", self))
    }
    pub fn expect_string(&self) -> Result<&str, String> {
        self.as_string()
            .ok_or_else(|| format!("expected String, got {:?}", self))
    }
    pub fn expect_bool(&self) -> Result<bool, String> {
        self.as_bool()
            .ok_or_else(|| format!("expected Bool, got {:?}", self))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Liternal::Int(_))
    }
    pub fn is_uint(&self) -> bool {
        matches!(self, Liternal::UInt(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Liternal::Float(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Liternal::String(_))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Liternal::Bool(_))
    }
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Liternal::Int(_) | Liternal::UInt(_) | Liternal::Float(_)
        )
    }
}
impl fmt::Display for Liternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Liternal::Int(v) => write!(f, "{}", v),
            Liternal::UInt(v) => write!(f, "{}", v),
            Liternal::Float(v) => write!(f, "{}", v),
            Liternal::String(v) => write!(f, "{}", v),
            Liternal::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl Debug for Liternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Liternal::Int(v) => write!(f, "{:?}", v),
            Liternal::UInt(v) => write!(f, "{:?}", v),
            Liternal::Float(v) => write!(f, "{:?}", v),
            Liternal::String(v) => write!(f, "String({:?})", v),
            Liternal::Bool(v) => write!(f, "Bool({})", v),
        }
    }
}

impl From<Int> for Liternal {
    fn from(v: Int) -> Self {
        Liternal::Int(v)
    }
}
impl From<UInt> for Liternal {
    fn from(v: UInt) -> Self {
        Liternal::UInt(v)
    }
}
impl From<Float> for Liternal {
    fn from(v: Float) -> Self {
        Liternal::Float(v)
    }
}
impl From<String> for Liternal {
    fn from(v: String) -> Self {
        Liternal::String(v)
    }
}
impl From<&str> for Liternal {
    fn from(v: &str) -> Self {
        Liternal::String(v.to_owned())
    }
}
impl From<bool> for Liternal {
    fn from(v: bool) -> Self {
        Liternal::Bool(v)
    }
}

impl From<i8> for Liternal {
    fn from(v: i8) -> Self {
        Liternal::Int(Int::Int8(v))
    }
}
impl From<i16> for Liternal {
    fn from(v: i16) -> Self {
        Liternal::Int(Int::Int16(v))
    }
}
impl From<i32> for Liternal {
    fn from(v: i32) -> Self {
        Liternal::Int(Int::Int32(v))
    }
}
impl From<i64> for Liternal {
    fn from(v: i64) -> Self {
        Liternal::Int(Int::Int64(v))
    }
}

impl From<u8> for Liternal {
    fn from(v: u8) -> Self {
        Liternal::UInt(UInt::UInt8(v))
    }
}
impl From<u16> for Liternal {
    fn from(v: u16) -> Self {
        Liternal::UInt(UInt::UInt16(v))
    }
}
impl From<u32> for Liternal {
    fn from(v: u32) -> Self {
        Liternal::UInt(UInt::UInt32(v))
    }
}
impl From<u64> for Liternal {
    fn from(v: u64) -> Self {
        Liternal::UInt(UInt::UInt64(v))
    }
}

impl From<f32> for Liternal {
    fn from(v: f32) -> Self {
        Liternal::Float(Float::Float32(v))
    }
}
impl From<f64> for Liternal {
    fn from(v: f64) -> Self {
        Liternal::Float(Float::Float64(v))
    }
}
