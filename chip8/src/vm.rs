pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_LEN: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

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
        memory[200..200 + rom.len()].copy_from_slice(rom);

        Self {
            ram: memory,
            pc: 0,
            idx_register: 0,
            delay: 0,
            sound: 0,
            v_registers: [0; 16],
            display: [false; DISPLAY_LEN],
        }
    }

    pub fn run(&self) -> Result<bool, String> {
        Ok(true)
    }
}
