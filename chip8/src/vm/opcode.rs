use std::convert::TryFrom;

use super::VmError;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Opcode {
    ClearScreen,
    // Jump(u16),
    // SetV(u8, u8),
    // AddV(u8, u8),
    // SetAddress(u16),
    // Display(u8, u8, u8),
}

impl TryFrom<u16> for Opcode {
    type Error = VmError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x00e0 => Ok(Self::ClearScreen),
            x => Err(VmError::InvalidOpcode(x)),
        }
    }
}
