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
pub struct Vm<R>
where
    R: Fn() -> u8,
{
    ram: [u8; 4096],
    pc: u16,
    i_register: u16,
    delay: u8,
    sound: u8,
    v_registers: [u8; 16],
    stack: Vec<u16>,
    randomize: R,

    is_waiting: bool,
    vx_after_wait: u8,

    pub display: [bool; DISPLAY_LEN],
    keys: [bool; 16],
}

#[cfg(test)]
impl<R> std::fmt::Display for Vm<R>
where
    R: Fn() -> u8,
{
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

impl<R> Vm<R>
where
    R: Fn() -> u8,
{
    pub fn new(rom: &[u8], randomize: R) -> Self {
        let mut memory = [0; 4096];
        memory[0x200..0x200 + rom.len()].copy_from_slice(rom);

        let mut res = Self {
            ram: memory,
            pc: 0x200,
            i_register: 0,
            delay: 0,
            sound: 0,
            v_registers: [0; 16],
            stack: Vec::with_capacity(16),
            display: [false; DISPLAY_LEN],
            keys: [false; 16],
            randomize,
            is_waiting: false,
            vx_after_wait: 0x0,
        };

        res.load_fonts();
        res
    }

    pub fn set_key(&mut self, key: u8, value: bool) -> Result<()> {
        if let Some(k) = self.keys.get_mut(key as usize) {
            *k = value;
            if value && self.is_waiting {
                self.is_waiting = false;
                self.v_registers[self.vx_after_wait as usize] = key;
            }
            Ok(())
        } else {
            Err(VmError::InvalidKey(key))
        }
    }

    pub fn tick_timers(&mut self) {
        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
    }

    pub fn sound(&self) -> u8 {
        self.sound
    }

    pub fn tick(&mut self) -> Result<()> {
        if self.is_waiting {
            return Ok(());
        }

        let raw_opcode = self.next_opcode()?;
        let opcode = Opcode::try_from(raw_opcode)?;

        match opcode {
            Opcode::ClearScreen => self.exec_clear_screen()?,
            Opcode::Ret => self.exec_return()?,
            Opcode::Jump(addr) => self.exec_jump_absolute(addr)?,
            Opcode::Call(addr) => self.exec_call(addr)?,
            Opcode::LoadVx(x, value) => self.exec_load_vx(x, value)?,
            Opcode::SkipIfEq(x, value) => self.exec_skip_if_equal(x, value)?,
            Opcode::SkipIfNeq(x, value) => self.exec_skip_if_not_equal(x, value)?,
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
            Opcode::SkipNeqVxVy(x, y) => self.exec_skip_if_not_equal_vx_vy(x, y)?,
            Opcode::LoadI(addr) => self.exec_load_i(addr)?,
            Opcode::JumpOffset(addr) => self.exec_jump_offset(addr)?,
            Opcode::Rand(x, value) => self.exec_rand(x, value)?,
            Opcode::Display(x, y, rows) => self.exec_display(x, y, rows)?,
            Opcode::SkipIfKey(x) => self.exec_skip_if_key(x)?,
            Opcode::SkipIfNotKey(x) => self.exec_skip_if_not_key(x)?,
            Opcode::LoadDelay(x) => self.exec_load_delay(x)?,
            Opcode::WaitForKey(x) => self.exec_wait_for_key(x)?,
            Opcode::StoreDelay(x) => self.exec_store_delay(x)?,
            Opcode::StoreSound(x) => self.exec_store_sound(x)?,
            Opcode::AddI(x) => self.exec_add_i(x)?,
            Opcode::LoadDigit(x) => self.exec_load_digit(x)?,
            Opcode::Bcd(x) => self.exec_bcd(x)?,
            Opcode::StoreRegisters(x) => self.exec_store_registers(x)?,
            Opcode::LoadRegisters(x) => self.exec_load_registers(x)?,
            Opcode::NoOp => {}
            _ => todo!(),
        };

        Ok(())
    }

    fn load_fonts(&mut self) {
        let numbers = [
            [0xf0, 0x90, 0x90, 0x90, 0xf0], // 0
            [0x20, 0x60, 0x20, 0x20, 0x70], // 1
            [0xf0, 0x10, 0xf0, 0x80, 0xf0], // 2
            [0xf0, 0x10, 0xf0, 0x10, 0xf0], // 3
            [0x90, 0x90, 0xf0, 0x10, 0x10], // 4
            [0xf0, 0x80, 0xf0, 0x10, 0x10], // 5
            [0xf0, 0x80, 0xf0, 0x90, 0xf0], // 6
            [0xf0, 0x10, 0x20, 0x40, 0x40], // 7
            [0xf0, 0x90, 0xf0, 0x90, 0xf0], // 8
            [0xf0, 0x90, 0xf0, 0x10, 0xf0], // 9
            [0xf0, 0x90, 0xf0, 0x90, 0x90], // A
            [0xe0, 0x90, 0xe0, 0x90, 0xe0], // B
            [0xf0, 0x80, 0x80, 0x80, 0xf0], // C
            [0xe0, 0x90, 0x90, 0x90, 0xe0], // D
            [0xf0, 0x80, 0xf0, 0x80, 0xf0], // E
            [0xf0, 0x80, 0xf0, 0x80, 0x80], // F
        ]
        .as_flattened();
        self.ram[0x00..numbers.len()].copy_from_slice(&numbers);
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

    #[inline]
    fn write_byte_at(&mut self, addr: u16, value: u8) -> Result<()> {
        let slot = self
            .ram
            .get_mut(addr as usize)
            .ok_or(VmError::InvalidAddress(addr))?;
        *slot = value;

        Ok(())
    }

    #[inline]
    fn read_byte_at(&mut self, addr: u16) -> Result<u8> {
        self.ram
            .get(addr as usize)
            .copied()
            .ok_or(VmError::InvalidAddress(addr))
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

    fn exec_skip_if_not_equal_vx_vy(&mut self, vx: u8, vy: u8) -> Result<()> {
        if self.v_registers[vx as usize] != self.v_registers[vy as usize] {
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

    fn exec_rand(&mut self, vx: u8, value: u8) -> Result<()> {
        self.v_registers[vx as usize] = (self.randomize)() & value;
        Ok(())
    }

    fn exec_skip_if_key(&mut self, vx: u8) -> Result<()> {
        let state = self.get_vx_key(vx)?;
        if state {
            self.pc += 2;
        }
        Ok(())
    }

    fn exec_skip_if_not_key(&mut self, vx: u8) -> Result<()> {
        let state = self.get_vx_key(vx)?;
        if !state {
            self.pc += 2;
        }
        Ok(())
    }

    #[inline]
    fn get_vx_key(&self, vx: u8) -> Result<bool> {
        let key = self.v_registers[vx as usize];
        self.keys
            .get(key as usize)
            .copied()
            .ok_or(VmError::InvalidKey(key))
    }

    fn exec_wait_for_key(&mut self, vx: u8) -> Result<()> {
        self.vx_after_wait = vx;
        self.is_waiting = true;

        Ok(())
    }

    fn exec_load_delay(&mut self, vx: u8) -> Result<()> {
        self.v_registers[vx as usize] = self.delay;
        Ok(())
    }

    fn exec_store_delay(&mut self, vx: u8) -> Result<()> {
        self.delay = self.v_registers[vx as usize];
        Ok(())
    }

    fn exec_store_sound(&mut self, vx: u8) -> Result<()> {
        self.sound = self.v_registers[vx as usize];
        Ok(())
    }

    fn exec_store_registers(&mut self, vx: u8) -> Result<()> {
        for i in 0..=vx as usize {
            self.write_byte_at(self.i_register, self.v_registers[i])?;
            self.i_register += 1;
        }

        Ok(())
    }

    fn exec_load_registers(&mut self, vx: u8) -> Result<()> {
        for i in 0..=vx as usize {
            self.v_registers[i] = self.read_byte_at(self.i_register)?;
            self.i_register += 1;
        }

        Ok(())
    }

    fn exec_call(&mut self, addr: u16) -> Result<()> {
        self.stack.push(self.pc);
        self.pc = addr;
        Ok(())
    }

    fn exec_return(&mut self) -> Result<()> {
        self.pc = self.stack.pop().ok_or(VmError::EmptyStack)?;
        Ok(())
    }

    fn exec_add_i(&mut self, vx: u8) -> Result<()> {
        self.i_register += self.v_registers[vx as usize] as u16;
        Ok(())
    }

    fn exec_bcd(&mut self, vx: u8) -> Result<()> {
        let mut value = self.v_registers[vx as usize];
        let hundreds = value / 100;
        value -= hundreds * 100;
        let tens = value / 10;
        value -= tens * 10;

        self.write_byte_at(self.i_register, hundreds)?;
        self.write_byte_at(self.i_register + 1, tens)?;
        self.write_byte_at(self.i_register + 2, value)?;

        Ok(())
    }

    fn exec_load_digit(&mut self, vx: u8) -> Result<()> {
        let nibble = self.v_registers[vx as usize] & 0x0f;
        let addr = nibble as u16 * 5;
        self.i_register = addr;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any_vm(rom: &[u8]) -> Vm<fn() -> u8> {
        Vm::new(&rom, || 0x00)
    }

    #[test]
    fn set_key_updates_value() {
        let mut vm = any_vm(&[]);
        let res = vm.set_key(0xf, true);

        assert!(res.is_ok());
        assert_eq!(vm.keys[0xf], true);
    }

    #[test]
    fn does_not_tick_when_waiting() {
        let mut vm = any_vm(&[0x00, 0xe0]);
        vm.is_waiting = true;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x200);
    }

    #[test]
    fn stops_wait_and_loads_vx_after_key_down() {
        let mut vm = any_vm(&[]);
        vm.is_waiting = true;
        vm.vx_after_wait = 0xb;

        let _ = vm.set_key(0xa, false);
        assert_eq!(vm.is_waiting, true);

        let _ = vm.set_key(0xa, true);
        assert_eq!(vm.is_waiting, false);
        assert_eq!(vm.v_registers[0xb], 0xa);
    }

    #[test]
    fn ticks_timers() {
        let mut vm = any_vm(&[]);
        vm.delay = 0x01;
        vm.sound = 0x02;

        vm.tick_timers();
        assert_eq!(vm.delay, 0x00);
        assert_eq!(vm.sound, 0x01);

        vm.tick_timers();
        assert_eq!(vm.delay, 0x00);
        assert_eq!(vm.sound, 0x00);
    }

    #[test]
    fn opcode_clear_screen() {
        let rom = [0x00, 0xe0];
        let mut vm = any_vm(&rom);
        vm.display = [true; DISPLAY_LEN];

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.display, [false; DISPLAY_LEN]);
    }

    #[test]
    fn opcode_jump_absolute() {
        let rom = [0x1a, 0xbc];
        let mut vm = any_vm(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0xabc);
    }

    #[test]
    fn opcode_load_vx() {
        let rom = [0x6a, 0xbc];
        let mut vm = any_vm(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0xa], 0xbc);
    }

    #[test]
    fn opcode_add_vx() {
        let rom = [0x7a, 0xbc];
        let mut vm = any_vm(&rom);
        vm.v_registers[0xa] = 0x11;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0xa], 0x11 + 0xbc);
    }

    #[test]
    fn opcode_skip_if_not_equal_vx_vy() {
        let rom = [0x90, 0x10];
        let mut vm = any_vm(&rom);
        vm.v_registers[0x0] = 0x00;
        vm.v_registers[0x1] = 0x01;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x204);

        vm.pc = 0x200;
        vm.v_registers[0x0] = 0x01;
        res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
    }

    #[test]
    fn opcode_load_i() {
        let rom = [0xaa, 0xbc];
        let mut vm = any_vm(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.i_register, 0x0abc);
    }

    #[test]
    fn opcode_display() {
        let rom = [0xd0, 0x14, 0xd0, 0x14, 0xff, 0x81, 0x81, 0xff];
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
        vm.v_registers[0x1] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0xab);
    }

    #[test]
    fn opcode_or_vx_vy() {
        let rom = [0x80, 0x11];
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
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
        let mut vm = any_vm(&rom);
        vm.v_registers[0x0] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x2ab);
    }

    #[test]
    fn opcode_rand() {
        let rom = [0xc0, 0x0f];
        let mut vm = Vm::new(&rom, || 0b1010_1010);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0b0000_1010);
    }

    #[test]
    fn opcode_skip_if_key() {
        let rom = [0xe0, 0x9e];
        let mut vm = any_vm(&rom);
        vm.v_registers[0x0] = 0xa;
        vm.keys[0xa] = true;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x204);

        vm.pc = 0x200;
        vm.keys[0xa] = false;
        res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
    }

    #[test]
    fn opcode_skip_if_not_key() {
        let rom = [0xe0, 0xa1];
        let mut vm = any_vm(&rom);
        vm.v_registers[0x0] = 0xa;
        vm.keys[0xa] = false;

        let mut res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x204);

        vm.pc = 0x200;
        vm.keys[0xa] = true;
        res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
    }

    #[test]
    fn opcode_wait_for_key() {
        let rom = [0xf0, 0x0a];
        let mut vm = any_vm(&rom);

        let res = vm.tick();
        assert!(res.is_ok());
        assert_eq!(vm.is_waiting, true);
        assert_eq!(vm.pc, 0x202);

        let _ = vm.set_key(0xa, true);
        assert_eq!(vm.is_waiting, false);
        assert_eq!(vm.v_registers[0x0], 0xa);
    }

    #[test]
    fn opcode_load_delay() {
        let rom = [0xf0, 0x07];
        let mut vm = any_vm(&rom);
        vm.delay = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.v_registers[0x0], 0xab);
    }

    #[test]
    fn opcode_store_delay() {
        let rom = [0xf0, 0x15];
        let mut vm = any_vm(&rom);
        vm.delay = 0x00;
        vm.v_registers[0x0] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.delay, 0xab);
    }

    #[test]
    fn opcode_store_sound() {
        let rom = [0xf0, 0x18];
        let mut vm = any_vm(&rom);
        vm.sound = 0x00;
        vm.v_registers[0x0] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.sound, 0xab);
    }

    #[test]
    fn opcode_store_registers() {
        let rom = [0xf2, 0x55];
        let mut vm = any_vm(&rom);
        vm.i_register = 0x300;
        vm.v_registers[0x0..0x03].copy_from_slice(&[0xa, 0xb, 0xc]);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.i_register, 0x303);
        assert_eq!(vm.ram[0x300..0x303], [0xa, 0xb, 0xc]);
    }

    #[test]
    fn opcode_load_registers() {
        let rom = [0xf2, 0x65];
        let mut vm = any_vm(&rom);
        vm.i_register = 0x300;
        vm.ram[0x300..0x303].copy_from_slice(&[0xa, 0xb, 0xc]);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.i_register, 0x303);
        assert_eq!(vm.v_registers[0..3], [0xa, 0xb, 0xc]);
    }

    #[test]
    fn opcode_call() {
        let rom = [0x23, 0x00];
        let mut vm = any_vm(&rom);

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pc, 0x300);
    }

    #[test]
    fn opcode_ret() {
        let rom = [0x00, 0xee];
        let mut vm = any_vm(&rom);
        vm.stack.push(0x300);

        let res = vm.tick();

        assert!(res.is_ok());
        assert!(vm.stack.is_empty());
        assert_eq!(vm.pc, 0x300);
    }

    #[test]
    fn opcode_bcd() {
        let rom = [0xf0, 0x33];
        let mut vm = any_vm(&rom);
        vm.i_register = 0x300;
        vm.v_registers[0x0] = 128;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.ram[0x300..0x303], [1, 2, 8]);
    }

    #[test]
    fn opcode_load_digit() {
        let rom = [0xf0, 0x29];
        let mut vm = any_vm(&rom);
        vm.v_registers[0x0] = 0xab;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.i_register, 0xb * 5);
    }

    #[test]
    fn opcode_add_i() {
        let rom = [0xf0, 0x1e];
        let mut vm = any_vm(&rom);
        vm.v_registers[0x0] = 0xab;
        vm.i_register = 0x300;

        let res = vm.tick();

        assert!(res.is_ok());
        assert_eq!(vm.pc, 0x202);
        assert_eq!(vm.i_register, 0x3ab);
    }
}
