use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Float {
    Float32(f32),
    Float64(f64),
}

impl Float {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Float::Float32(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Float::Float64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn expect_f32(&self) -> Result<f32, String> {
        self.as_f32()
            .ok_or_else(|| format!("expected Float32, got {:?}", self))
    }
    pub fn expect_f64(&self) -> Result<f64, String> {
        self.as_f64()
            .ok_or_else(|| format!("expected Float64, got {:?}", self))
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float32(v) => write!(f, "{}", v),
            Self::Float64(v) => write!(f, "{}", v),
        }
    }
}

impl fmt::Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float32(v) => write!(f, "Float32({})", v),
            Self::Float64(v) => write!(f, "Float64({})", v),
        }
    }
}

impl From<f32> for Float {
    fn from(v: f32) -> Self {
        Float::Float32(v)
    }
}
impl From<f64> for Float {
    fn from(v: f64) -> Self {
        Float::Float64(v)
    }
}
