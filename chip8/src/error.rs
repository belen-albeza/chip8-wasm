use wasm_bindgen::JsValue;

use std::error::Error as ErrorTrait;
use std::fmt;

pub use crate::vm::VmError;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    VmError(VmError),
    InvalidRom,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.source() {
            Some(err) => write!(f, "{}", err),
            None => match self {
                Self::InvalidRom => write!(f, "Invalid ROM"),
                _ => write!(f, "{:?}", self),
            },
        }
    }
}

impl ErrorTrait for Error {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        match self {
            Self::VmError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        JsValue::from(format!("{}", self))
    }
}

impl From<VmError> for Error {
    fn from(value: VmError) -> Self {
        Self::VmError(value)
    }
}
