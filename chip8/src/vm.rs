mod error;
mod opcode;

use std::convert::TryFrom;

pub use error::VmError;
use opcode::Opcode;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_LEN: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

pub type Result<T> = core::result::Result<T, VmError>;

#[derive(Debug, PartialEq, Clone)]
pub struct Vm {
    ram: [u8; 4096],
    pc: u16,
    idx_register: u16,
    delay: u8,
    sound: u8,
    v_registers: [u8; 16],
    pub display: [bool; DISPLAY_LEN],
}

impl Vm {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = [0; 4096];
        memory[0x200..0x200 + rom.len()].copy_from_slice(rom);

        Self {
            ram: memory,
            pc: 0x200,
            idx_register: 0,
            delay: 0,
            sound: 0,
            v_registers: [0; 16],
            display: [false; DISPLAY_LEN],
        }
    }

    pub fn run(&mut self) -> Result<bool> {
        loop {
            let shall_halt = self.tick()?;
            if shall_halt {
                break;
            }
        }

        Ok(true)
    }

    pub fn tick(&mut self) -> Result<bool> {
        let raw_opcode = self.next_opcode()?;
        let opcode = Opcode::try_from(raw_opcode)?;

        let shall_halt = match opcode {
            Opcode::ClearScreen => self.exec_clear_screen()?,
            Opcode::Jump(addr) => self.exec_jump_absolute(addr)?,
            _ => todo!(),
        };

        Ok(shall_halt)
    }

    fn next_opcode(&mut self) -> Result<u16> {
        let hi = self.read_byte()?;
        let lo = self.read_byte()?;
        let raw_opcode = u16::from_be_bytes([hi, lo]);

        Ok(raw_opcode)
    }

    fn read_byte(&mut self) -> Result<u8> {
        let res = self
            .ram
            .get(self.pc as usize)
            .copied()
            .ok_or(VmError::InvalidAddress(self.pc));
        self.pc += 1;
        res
    }

    fn exec_clear_screen(&mut self) -> Result<bool> {
        self.display = [false; DISPLAY_LEN];
        Ok(false)
    }

    fn exec_jump_absolute(&mut self, addr: u16) -> Result<bool> {
        self.pc = addr;
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_clear_screen() {
        let rom = [0x00, 0xe0];
        let mut vm = Vm::new(&rom);
        vm.display = [true; DISPLAY_LEN];

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.display, [false; DISPLAY_LEN]);
    }

    #[test]
    fn opcode_jump_absolute() {
        let rom = [0x1a, 0xbc];
        let mut vm = Vm::new(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0xabc);
    }
}
