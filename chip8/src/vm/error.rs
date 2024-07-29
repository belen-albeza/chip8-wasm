use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidAddress(u16),
    InvalidOpcode(u16),
    InvalidKey(u8),
    EmptyStack,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress(addr) => write!(f, "Invalid address: {:#06x}", addr),
            Self::InvalidOpcode(opcode) => write!(f, "Invalid opcode: {:#06x}", opcode),
            Self::InvalidKey(id) => write!(f, "Invalid key: {:#04x}", id),
            Self::EmptyStack => write!(f, "Stack is empty"),
        }
    }
}
impl error::Error for VmError {}
