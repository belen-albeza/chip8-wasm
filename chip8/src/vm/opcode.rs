use std::convert::TryFrom;

use super::VmError;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Opcode {
    NoOp,
    ClearScreen,
    Jump(u16),
    SkipIfEq(u8, u8),
    SkipIfNotEq(u8, u8),
    SkipEqVxVy(u8, u8),
    LoadVx(u8, u8),
    AddVx(u8, u8),
    LoadVxVy(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    LoadI(u16),
    Display(u8, u8, u8),
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
            (0x3, x, _, _) => Ok(Self::SkipIfEq(x, kk)),
            (0x4, x, _, _) => Ok(Self::SkipIfNotEq(x, kk)),
            (0x5, x, y, 0) => Ok(Self::SkipEqVxVy(x, y)),
            (0x6, x, _, _) => Ok(Self::LoadVx(x, kk)),
            (0x7, x, _, _) => Ok(Self::AddVx(x, kk)),
            (0x8, x, y, 0) => Ok(Self::LoadVxVy(x, y)),
            (0x8, x, y, 1) => Ok(Self::Or(x, y)),
            (0x8, x, y, 2) => Ok(Self::And(x, y)),
            (0x8, x, y, 3) => Ok(Self::Xor(x, y)),
            (0x8, x, y, 4) => Ok(Self::Add(x, y)),
            (0xa, _, _, _) => Ok(Self::LoadI(nnn)),
            (0xd, x, y, n) => Ok(Self::Display(x, y, n)),
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
        assert_eq!(Opcode::try_from(0x3abc), Ok(Opcode::SkipIfEq(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x4abc), Ok(Opcode::SkipIfNotEq(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x5ab0), Ok(Opcode::SkipEqVxVy(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x6abc), Ok(Opcode::LoadVx(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x7abc), Ok(Opcode::AddVx(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x8ab0), Ok(Opcode::LoadVxVy(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab1), Ok(Opcode::Or(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab2), Ok(Opcode::And(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab3), Ok(Opcode::Xor(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab4), Ok(Opcode::Add(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0xaabc), Ok(Opcode::LoadI(0x0abc)));
        assert_eq!(Opcode::try_from(0xdabc), Ok(Opcode::Display(0xa, 0xb, 0xc)));
    }
}
