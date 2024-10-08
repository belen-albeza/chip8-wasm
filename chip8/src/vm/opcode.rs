use std::convert::TryFrom;

use super::VmError;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Opcode {
    NoOp,
    ClearScreen,
    Ret,
    Jump(u16),
    Call(u16),
    SkipIfEq(u8, u8),
    SkipIfNeq(u8, u8),
    SkipEqVxVy(u8, u8),
    LoadVx(u8, u8),
    AddVx(u8, u8),
    LoadVxVy(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    Sub(u8, u8),
    ShiftR(u8, u8),
    SubN(u8, u8),
    ShiftL(u8, u8),
    SkipNeqVxVy(u8, u8),
    LoadI(u16),
    JumpOffset(u16),
    Rand(u8, u8),
    Display(u8, u8, u8),
    SkipIfKey(u8),
    SkipIfNotKey(u8),
    LoadDelay(u8),
    WaitForKey(u8),
    StoreDelay(u8),
    StoreSound(u8),
    AddI(u8),
    LoadDigit(u8),
    Bcd(u8),
    StoreRegisters(u8),
    LoadRegisters(u8),
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
            (0x0, 0x0, 0xe, 0xe) => Ok(Self::Ret),
            (0x0, _, _, _) => Ok(Self::NoOp),
            (0x1, _, _, _) => Ok(Self::Jump(nnn)),
            (0x2, _, _, _) => Ok(Self::Call(nnn)),
            (0x3, x, _, _) => Ok(Self::SkipIfEq(x, kk)),
            (0x4, x, _, _) => Ok(Self::SkipIfNeq(x, kk)),
            (0x5, x, y, 0) => Ok(Self::SkipEqVxVy(x, y)),
            (0x6, x, _, _) => Ok(Self::LoadVx(x, kk)),
            (0x7, x, _, _) => Ok(Self::AddVx(x, kk)),
            (0x8, x, y, 0x0) => Ok(Self::LoadVxVy(x, y)),
            (0x8, x, y, 0x1) => Ok(Self::Or(x, y)),
            (0x8, x, y, 0x2) => Ok(Self::And(x, y)),
            (0x8, x, y, 0x3) => Ok(Self::Xor(x, y)),
            (0x8, x, y, 0x4) => Ok(Self::Add(x, y)),
            (0x8, x, y, 0x5) => Ok(Self::Sub(x, y)),
            (0x8, x, y, 0x6) => Ok(Self::ShiftR(x, y)),
            (0x8, x, y, 0x7) => Ok(Self::SubN(x, y)),
            (0x8, x, y, 0xe) => Ok(Self::ShiftL(x, y)),
            (0x9, x, y, 0) => Ok(Self::SkipNeqVxVy(x, y)),
            (0xa, _, _, _) => Ok(Self::LoadI(nnn)),
            (0xb, _, _, _) => Ok(Self::JumpOffset(nnn)),
            (0xc, x, _, _) => Ok(Self::Rand(x, kk)),
            (0xd, x, y, n) => Ok(Self::Display(x, y, n)),
            (0xe, x, 0x9, 0xe) => Ok(Self::SkipIfKey(x)),
            (0xe, x, 0xa, 0x1) => Ok(Self::SkipIfNotKey(x)),
            (0xf, x, 0x0, 0x7) => Ok(Self::LoadDelay(x)),
            (0xf, x, 0x0, 0xa) => Ok(Self::WaitForKey(x)),
            (0xf, x, 0x1, 0x5) => Ok(Self::StoreDelay(x)),
            (0xf, x, 0x1, 0x8) => Ok(Self::StoreSound(x)),
            (0xf, x, 0x1, 0xe) => Ok(Self::AddI(x)),
            (0xf, x, 0x2, 0x9) => Ok(Self::LoadDigit(x)),
            (0xf, x, 0x3, 0x3) => Ok(Self::Bcd(x)),
            (0xf, x, 0x5, 0x5) => Ok(Self::StoreRegisters(x)),
            (0xf, x, 0x6, 0x5) => Ok(Self::LoadRegisters(x)),
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
        assert_eq!(Opcode::try_from(0x00ee), Ok(Opcode::Ret));
        assert_eq!(Opcode::try_from(0x0abc), Ok(Opcode::NoOp));
        assert_eq!(Opcode::try_from(0x1abc), Ok(Opcode::Jump(0x0abc)));
        assert_eq!(Opcode::try_from(0x2abc), Ok(Opcode::Call(0x0abc)));
        assert_eq!(Opcode::try_from(0x3abc), Ok(Opcode::SkipIfEq(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x4abc), Ok(Opcode::SkipIfNeq(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x5ab0), Ok(Opcode::SkipEqVxVy(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x6abc), Ok(Opcode::LoadVx(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x7abc), Ok(Opcode::AddVx(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0x8ab0), Ok(Opcode::LoadVxVy(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab1), Ok(Opcode::Or(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab2), Ok(Opcode::And(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab3), Ok(Opcode::Xor(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab4), Ok(Opcode::Add(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab5), Ok(Opcode::Sub(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab6), Ok(Opcode::ShiftR(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8ab7), Ok(Opcode::SubN(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x8abe), Ok(Opcode::ShiftL(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0x9ab0), Ok(Opcode::SkipNeqVxVy(0xa, 0xb)));
        assert_eq!(Opcode::try_from(0xaabc), Ok(Opcode::LoadI(0x0abc)));
        assert_eq!(Opcode::try_from(0xbabc), Ok(Opcode::JumpOffset(0x0abc)));
        assert_eq!(Opcode::try_from(0xcabc), Ok(Opcode::Rand(0xa, 0xbc)));
        assert_eq!(Opcode::try_from(0xdabc), Ok(Opcode::Display(0xa, 0xb, 0xc)));
        assert_eq!(Opcode::try_from(0xea9e), Ok(Opcode::SkipIfKey(0xa)));
        assert_eq!(Opcode::try_from(0xeaa1), Ok(Opcode::SkipIfNotKey(0xa)));
        assert_eq!(Opcode::try_from(0xfa07), Ok(Opcode::LoadDelay(0xa)));
        assert_eq!(Opcode::try_from(0xfa0a), Ok(Opcode::WaitForKey(0xa)));
        assert_eq!(Opcode::try_from(0xfa15), Ok(Opcode::StoreDelay(0xa)));
        assert_eq!(Opcode::try_from(0xfa18), Ok(Opcode::StoreSound(0xa)));
        assert_eq!(Opcode::try_from(0xfa1e), Ok(Opcode::AddI(0xa)));
        assert_eq!(Opcode::try_from(0xfa29), Ok(Opcode::LoadDigit(0xa)));
        assert_eq!(Opcode::try_from(0xfa33), Ok(Opcode::Bcd(0xa)));
        assert_eq!(Opcode::try_from(0xfa55), Ok(Opcode::StoreRegisters(0xa)));
        assert_eq!(Opcode::try_from(0xfa65), Ok(Opcode::LoadRegisters(0xa)));
    }
}
