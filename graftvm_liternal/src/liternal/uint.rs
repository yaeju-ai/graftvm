use std::fmt;
use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub enum UInt {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
}

impl UInt {
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            UInt::UInt8(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_u16(&self) -> Option<u16> {
        match self {
            UInt::UInt16(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            UInt::UInt32(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            UInt::UInt64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn expect_u8(&self) -> Result<u8, String> {
        self.as_u8()
            .ok_or_else(|| format!("expected UInt8, got {:?}", self))
    }
    pub fn expect_u16(&self) -> Result<u16, String> {
        self.as_u16()
            .ok_or_else(|| format!("expected UInt16, got {:?}", self))
    }
    pub fn expect_u32(&self) -> Result<u32, String> {
        self.as_u32()
            .ok_or_else(|| format!("expected UInt32, got {:?}", self))
    }
    pub fn expect_u64(&self) -> Result<u64, String> {
        self.as_u64()
            .ok_or_else(|| format!("expected UInt64, got {:?}", self))
    }
}

impl fmt::Display for UInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UInt8(v) => write!(f, "{}", v),
            Self::UInt16(v) => write!(f, "{}", v),
            Self::UInt32(v) => write!(f, "{}", v),
            Self::UInt64(v) => write!(f, "{}", v),
        }
    }
}

impl Debug for UInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UInt8(v) => write!(f, "UInt8({})", v),
            Self::UInt16(v) => write!(f, "UInt16({})", v),
            Self::UInt32(v) => write!(f, "UInt32({})", v),
            Self::UInt64(v) => write!(f, "UInt64({})", v),
        }
    }
}

impl From<u8> for UInt {
    fn from(v: u8) -> Self {
        UInt::UInt8(v)
    }
}
impl From<u16> for UInt {
    fn from(v: u16) -> Self {
        UInt::UInt16(v)
    }
}
impl From<u32> for UInt {
    fn from(v: u32) -> Self {
        UInt::UInt32(v)
    }
}
impl From<u64> for UInt {
    fn from(v: u64) -> Self {
        UInt::UInt64(v)
    }
}
