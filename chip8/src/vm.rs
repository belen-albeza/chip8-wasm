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
    i_register: u16,
    delay: u8,
    sound: u8,
    v_registers: [u8; 16],
    pub display: [bool; DISPLAY_LEN],
}

#[cfg(test)]
impl std::fmt::Display for Vm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    pub fn tick(&mut self) -> Result<()> {
        let raw_opcode = self.next_opcode()?;
        let opcode = Opcode::try_from(raw_opcode)?;

        match opcode {
            Opcode::ClearScreen => self.exec_clear_screen()?,
            Opcode::Jump(addr) => self.exec_jump_absolute(addr)?,
            Opcode::LoadVx(x, value) => self.exec_load_vx(x, value)?,
            Opcode::SkipIfEq(x, value) => self.exec_skip_if_equal(x, value)?,
            Opcode::SkipIfNotEq(x, value) => self.exec_skip_if_not_equal(x, value)?,
            Opcode::SkipEqVxVy(x, y) => self.exec_skip_if_equal_vx_vy(x, y)?,
            Opcode::AddVx(x, value) => self.exec_add_vx(x, value)?,
            Opcode::LoadVxVy(x, y) => self.exec_load_vx_vy(x, y)?,
            Opcode::Or(x, y) => self.exec_or_vx_vy(x, y)?,
            Opcode::And(x, y) => self.exec_and_vx_vy(x, y)?,
            Opcode::Xor(x, y) => self.exec_xor_vx_vy(x, y)?,
            Opcode::Add(x, y) => self.exec_add_vx_vy(x, y)?,
            Opcode::Sub(x, y) => self.exec_sub_vx_vy(x, y)?,
            Opcode::ShiftR(x, y) => self.exec_shift_right(x, y)?,
            Opcode::SubN(x, y) => self.exec_subn_vy_vx(x, y)?,
            Opcode::ShiftL(x, y) => self.exec_shift_left(x, y)?,
            Opcode::LoadI(addr) => self.exec_load_i(addr)?,
            Opcode::JumpOffset(addr) => self.exec_jump_offset(addr)?,
            Opcode::Display(x, y, rows) => self.exec_display(x, y, rows)?,
            Opcode::NoOp => {}
            _ => todo!(),
        };

        Ok(())
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

    fn exec_clear_screen(&mut self) -> Result<()> {
        self.display = [false; DISPLAY_LEN];
        Ok(())
    }

    fn exec_jump_absolute(&mut self, addr: u16) -> Result<()> {
        self.pc = addr;
        Ok(())
    }

    fn exec_load_vx(&mut self, x: u8, value: u8) -> Result<()> {
        self.v_registers[x as usize] = value;
        Ok(())
    }

    fn exec_add_vx(&mut self, x: u8, value: u8) -> Result<()> {
        self.v_registers[x as usize] = self.v_registers[x as usize].wrapping_add(value);
        Ok(())
    }

    fn exec_load_i(&mut self, addr: u16) -> Result<()> {
        self.i_register = addr;
        Ok(())
    }

    fn exec_display(&mut self, vx: u8, vy: u8, rows: u8) -> Result<()> {
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

        Ok(())
    }

    #[inline]
    fn put_pixel(&mut self, x: u8, y: u8) -> bool {
        let x = x as usize % DISPLAY_WIDTH;
        let y = y as usize % DISPLAY_HEIGHT;
        let i = y * DISPLAY_WIDTH + x;

        let erased = self.display[i];
        self.display[i] = !self.display[i];

        return erased;
    }

    fn exec_skip_if_equal(&mut self, vx: u8, value: u8) -> Result<()> {
        if self.v_registers[vx as usize] == value {
            self.pc += 2;
        }

        Ok(())
    }

    fn exec_skip_if_not_equal(&mut self, vx: u8, value: u8) -> Result<()> {
        if self.v_registers[vx as usize] != value {
            self.pc += 2;
        }

        Ok(())
    }

    fn exec_skip_if_equal_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        if self.v_registers[vx as usize] == self.v_registers[vy as usize] {
            self.pc += 2;
        }

        Ok(())
    }

    fn exec_load_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        self.v_registers[vx as usize] = self.v_registers[vy as usize];
        Ok(())
    }

    fn exec_or_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        self.v_registers[vx as usize] |= self.v_registers[vy as usize];
        Ok(())
    }

    fn exec_and_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        self.v_registers[vx as usize] &= self.v_registers[vy as usize];
        Ok(())
    }

    fn exec_xor_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        self.v_registers[vx as usize] ^= self.v_registers[vy as usize];
        Ok(())
    }

    fn exec_add_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        let (value, carry) =
            self.v_registers[vx as usize].overflowing_add(self.v_registers[vy as usize]);
        self.v_registers[vx as usize] = value;
        self.v_registers[0xf] = if carry { 0x01 } else { 0x00 };

        Ok(())
    }

    fn exec_sub_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        let (value, overflow) =
            self.v_registers[vx as usize].overflowing_sub(self.v_registers[vy as usize]);
        self.v_registers[vx as usize] = value;
        self.v_registers[0xf] = if overflow { 0x00 } else { 0x01 };

        Ok(())
    }

    fn exec_shift_right(&mut self, vx: u8, vy: u8) -> Result<()> {
        let y = self.v_registers[vy as usize];
        let shifted_out = y & 0b0000_0001;
        self.v_registers[vx as usize] = y >> 1;
        self.v_registers[0xf] = shifted_out;

        Ok(())
    }

    fn exec_subn_vy_vx(&mut self, vx: u8, vy: u8) -> Result<()> {
        let (value, overflow) =
            self.v_registers[vy as usize].overflowing_sub(self.v_registers[vx as usize]);
        self.v_registers[vx as usize] = value;
        self.v_registers[0xf] = if overflow { 0x00 } else { 0x01 };

        Ok(())
    }

    fn exec_shift_left(&mut self, vx: u8, vy: u8) -> Result<()> {
        let y = self.v_registers[vy as usize];
        let shifted_out = (y & 0b1000_0000) >> 7;
        self.v_registers[vx as usize] = y << 1;
        self.v_registers[0xf] = shifted_out;

        Ok(())
    }

    fn exec_jump_offset(&mut self, addr: u16) -> Result<()> {
        self.pc = addr + self.v_registers[0x0] as u16;
        Ok(())
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
        vm.i_register = 0x204;
        vm.v_registers[0x0] = 0x01;
        vm.v_registers[0x1] = 0x00;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
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
        assert_eq!(vm.pc, 0x204);
        assert_eq!(vm.display, [false; DISPLAY_LEN]);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }

    #[test]
    fn opcode_display_with_wrap() {
        let rom = [0xd0, 0x12, 0xff, 0xff];
        let mut vm = Vm::new(&rom);
        vm.i_register = 0x202;
        vm.v_registers[0x0] = 60;
        vm.v_registers[0x1] = 31;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.display[0..4], [true; 4]);
        assert_eq!(vm.display[60..64], [true; 4]);
        assert_eq!(
            vm.display[0 + 31 * DISPLAY_WIDTH..(4 + 31 * DISPLAY_WIDTH)],
            [true; 4]
        );
        assert_eq!(
            vm.display[60 + 31 * DISPLAY_WIDTH..(64 + 31 * DISPLAY_WIDTH)],
            [true; 4]
        );
    }

    #[test]
    fn opcode_skip_if_equal() {
        let rom = [0x30, 0xab];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0xab;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x204);

        vm.pc = 0x200;
        vm.v_registers[0x0] = 0x0;
        res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
    }

    #[test]
    fn opcode_skip_if_not_equal() {
        let rom = [0x40, 0xab];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x00;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x204);

        vm.pc = 0x200;
        vm.v_registers[0x0] = 0xab;
        res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
    }

    #[test]
    fn opcode_skip_if_equal_vx_vy() {
        let rom = [0x50, 0x10];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x00;
        vm.v_registers[0x1] = 0x00;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x204);

        vm.pc = 0x200;
        vm.v_registers[0x0] = 0xab;
        res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
    }

    #[test]
    fn opcode_load_vx_vy() {
        let rom = [0x80, 0x10];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x1] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0xab);
    }

    #[test]
    fn opcode_or_vx_vy() {
        let rom = [0x80, 0x11];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x0f;
        vm.v_registers[0x1] = 0b_0101_0101;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0b_0101_1111);
    }

    #[test]
    fn opcode_and_vx_vy() {
        let rom = [0x80, 0x12];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x0f;
        vm.v_registers[0x1] = 0b_0101_0101;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0b_0000_0101);
    }

    #[test]
    fn opcode_xor_vx_vy() {
        let rom = [0x80, 0x13];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x0f;
        vm.v_registers[0x1] = 0b_0101_0101;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0b_0101_1010);
    }

    #[test]
    fn opcode_add_vx_vy() {
        let rom = [0x80, 0x14];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x02;
        vm.v_registers[0x1] = 0x01;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0x03);
        assert_eq!(vm.v_registers[0xf], 0x00);
    }

    #[test]
    fn opcode_add_vx_vy_with_carry() {
        let rom = [0x80, 0x14];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0xff;
        vm.v_registers[0x1] = 0x01;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0x00);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }

    #[test]
    fn opcode_sub_vx_vy() {
        let rom = [0x80, 0x15];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x03;
        vm.v_registers[0x1] = 0x01;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0x02);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }

    #[test]
    fn opcode_sub_vx_vy_with_borrow() {
        let rom = [0x80, 0x15];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x00;
        vm.v_registers[0x1] = 0x01;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0xff);
        assert_eq!(vm.v_registers[0xf], 0x00);
    }

    #[test]
    fn opcode_shift_right() {
        let rom = [0x80, 0x16];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x00;
        vm.v_registers[0x1] = 0b_0000_0011;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0x01);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }

    #[test]
    fn opcode_subn_vy_vx() {
        let rom = [0x80, 0x17];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x01;
        vm.v_registers[0x1] = 0x03;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0x02);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }

    #[test]
    fn opcode_shift_left() {
        let rom = [0x80, 0x1e];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0x00;
        vm.v_registers[0x1] = 0b_1000_0001;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0x02);
        assert_eq!(vm.v_registers[0xf], 0x01);
    }

    #[test]
    fn opcode_jump_offset() {
        let rom = [0xb2, 0x00];
        let mut vm = Vm::new(&rom);
        vm.v_registers[0x0] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x2ab);
    }
}
