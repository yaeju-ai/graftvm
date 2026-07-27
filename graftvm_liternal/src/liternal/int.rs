use std::fmt::Debug;
use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Int {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
}

impl Int {
    pub fn as_i8(&self) -> Option<i8> {
        match self {
            Int::Int8(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_i16(&self) -> Option<i16> {
        match self {
            Int::Int16(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Int::Int32(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Int::Int64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn expect_i8(&self) -> Result<i8, String> {
        self.as_i8()
            .ok_or_else(|| format!("expected Int8, got {:?}", self))
    }
    pub fn expect_i16(&self) -> Result<i16, String> {
        self.as_i16()
            .ok_or_else(|| format!("expected Int16, got {:?}", self))
    }
    pub fn expect_i32(&self) -> Result<i32, String> {
        self.as_i32()
            .ok_or_else(|| format!("expected Int32, got {:?}", self))
    }
    pub fn expect_i64(&self) -> Result<i64, String> {
        self.as_i64()
            .ok_or_else(|| format!("expected Int64, got {:?}", self))
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int8(v) => write!(f, "{}", v),
            Self::Int16(v) => write!(f, "{}", v),
            Self::Int32(v) => write!(f, "{}", v),
            Self::Int64(v) => write!(f, "{}", v),
        }
    }
}

impl Debug for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int8(v) => write!(f, "Int8({})", v),
            Self::Int16(v) => write!(f, "Int16({})", v),
            Self::Int32(v) => write!(f, "Int32({})", v),
            Self::Int64(v) => write!(f, "Int64({})", v),
        }
    }
}

impl From<i8> for Int {
    fn from(v: i8) -> Self {
        Int::Int8(v)
    }
}
impl From<i16> for Int {
    fn from(v: i16) -> Self {
        Int::Int16(v)
    }
}
impl From<i32> for Int {
    fn from(v: i32) -> Self {
        Int::Int32(v)
    }
}
impl From<i64> for Int {
    fn from(v: i64) -> Self {
        Int::Int64(v)
    }
}
