use std::convert::TryFrom;

use super::VmError;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Opcode {
    NoOp,
    ClearScreen,
    Jump(u16),
    LoadVx(u8, u8),
    AddVx(u8, u8),
    LoadI(u16),
    // Display(u8, u8, u8),
}

impl TryFrom<u16> for Opcode {
    type Error = VmError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let nibbles = (
            ((value & 0xf000) >> 12) as u8,
            ((value & 0x0f00) >> 8) as u8,
            ((value & 0x00f0) >> 4) as u8,
            (value & 0x000f) as u8,
        );

        let nnn = (value & 0x0fff) as u16;
        let kk = (value & 0x00ff) as u8;

        match nibbles {
            (0x0, 0x0, 0xe, 0x0) => Ok(Self::ClearScreen),
            (0x0, _, _, _) => Ok(Self::NoOp),
            (0x1, _, _, _) => Ok(Self::Jump(nnn)),
            (0x6, x, _, _) => Ok(Self::LoadVx(x, kk)),
            (0x7, x, _, _) => Ok(Self::AddVx(x, kk)),
            (0xa, x, _, _) => Ok(Self::LoadI(nnn)),
            _ => Err(VmError::InvalidOpcode(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_short() {
        assert_eq!(Opcode::try_from(0x00e0), Ok(Opcode::ClearScreen));
        assert_eq!(Opcode::try_from(0x0abc), Ok(Opcode::NoOp));
        assert_eq!(Opcode::try_from(0x1abc), Ok(Opcode::Jump(0x0abc)));
        assert_eq!(Opcode::try_from(0x6abc), Ok(Opcode::LoadVx(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x7abc), Ok(Opcode::AddVx(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0xaabc), Ok(Opcode::LoadI(0x0abc)));
    }
}
