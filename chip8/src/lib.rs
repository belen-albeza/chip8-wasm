mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Vm {
    ram: [u8; 4096],
    pc: u16,
    idx_register: u16,
    delay: u8,
    sound: u8,
    v_registers: [u8; 16],
}

#[wasm_bindgen]
impl Vm {
    #[wasm_bindgen(constructor)]
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
        }
    }

    #[wasm_bindgen]
    pub fn run(&self) -> Result<(), String> {
        Ok(())
    }
}

#[wasm_bindgen(js_name=loadRom)]
pub fn load_rom(rom: &[u8]) -> Result<Vm, String> {
    utils::set_panic_hook();

    if rom.len() > 4096 - 200 {
        return Err("ROM does not fit into memory".to_string());
    }

    Ok(Vm::new(rom))
}
