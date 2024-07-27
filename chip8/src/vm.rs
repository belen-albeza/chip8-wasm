mod error;
mod opcode;

use std::convert::TryFrom;
use std::fmt;

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
    i_register: u16,
    delay: u8,
    sound: u8,
    v_registers: [u8; 16],
    pub display: [bool; DISPLAY_LEN],
}

#[cfg(test)]
impl fmt::Display for Vm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                write!(
                    f,
                    "{}",
                    if self.display[y * DISPLAY_WIDTH + x] {
                        "*"
                    } else {
                        " "
                    }
                )?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Vm {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = [0; 4096];
        memory[0x200..0x200 + rom.len()].copy_from_slice(rom);

        Self {
            ram: memory,
            pc: 0x200,
            i_register: 0,
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
            Opcode::LoadVx(x, value) => self.exec_load_vx(x, value)?,
            Opcode::AddVx(x, value) => self.exec_add_vx(x, value)?,
            Opcode::LoadI(addr) => self.exec_load_i(addr)?,
            Opcode::Display(x, y, rows) => self.exec_display(x, y, rows)?,
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

    fn exec_load_vx(&mut self, x: u8, value: u8) -> Result<bool> {
        self.v_registers[x as usize] = value;
        Ok(false)
    }

    fn exec_add_vx(&mut self, x: u8, value: u8) -> Result<bool> {
        self.v_registers[x as usize] = self.v_registers[x as usize].wrapping_add(value);
        Ok(false)
    }

    fn exec_load_i(&mut self, addr: u16) -> Result<bool> {
        self.i_register = addr;
        Ok(false)
    }

    fn exec_display(&mut self, vx: u8, vy: u8, rows: u8) -> Result<bool> {
        self.v_registers[0xf] = 0x00;

        let sprite_x = self.v_registers[vx as usize];
        let sprite_y = self.v_registers[vy as usize];

        let addr = self.i_register as usize;
        let sprite = self.ram[addr..addr + rows as usize].to_vec();

        for row in 0..rows {
            for col in 0..8 {
                let pixel = (sprite[row as usize] & (0x80 >> col)) != 0;
                if !pixel {
                    continue;
                }

                let did_erase = self.put_pixel(sprite_x + col, sprite_y + row);
                if did_erase {
                    self.v_registers[0xf] = 0x01;
                }
            }
        }

        Ok(false)
    }

    #[inline]
    fn put_pixel(&mut self, x: u8, y: u8) -> bool {
        let i = y as usize * DISPLAY_WIDTH + x as usize;
        let erased = self.display[i];
        self.display[i] = !self.display[i];

        return erased;
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
        assert_eq!(vm.pc, 0x202);
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

    #[test]
    fn opcode_load_vx() {
        let rom = [0x6a, 0xbc];
        let mut vm = Vm::new(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0xa], 0xbc);
    }

    #[test]
    fn opcode_add_vx() {
        let rom = [0x7a, 0xbc];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0xa] = 0x11;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0xa], 0x11 + 0xbc);
    }

    #[test]
    fn opcode_load_i() {
        let rom = [0xaa, 0xbc];
        let mut vm = Vm::new(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.i_register, 0x0abc);
    }

    #[test]
    fn opcode_display() {
        let rom = [0xd0, 0x14, 0xd0, 0x14, 0xff, 0x81, 0x81, 0xff];
        let mut vm = Vm::new(&rom);
        vm.i_register = 0x200 + 0x04;
        vm.v_registers[0x0] = 0x01;
        vm.v_registers[0x1] = 0x00;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.v_registers[0xf], 0x00);
        assert_eq!(vm.display[1..9], [true; 8]);
        assert_eq!(
            vm.display[(64 + 1)..(64 + 9)],
            [true, false, false, false, false, false, false, true]
        );
        assert_eq!(
            vm.display[(64 * 2 + 1)..(64 * 2 + 9)],
            [true, false, false, false, false, false, false, true]
        );
        assert_eq!(vm.display[(64 * 3 + 1)..(64 * 3 + 9)], [true; 8]);

        res = vm.tick();
        assert!(res.is_ok());
        assert_eq!(vm.display, [false; DISPLAY_LEN]);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }
}
