use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidAddress(u16),
    InvalidOpcode(u16),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress(addr) => write!(f, "Invalid address: {:#06x}", addr),
            Self::InvalidOpcode(opcode) => write!(f, "Invalid opcode: {:#06x}", opcode),
        }
    }
}
impl error::Error for VmError {}
