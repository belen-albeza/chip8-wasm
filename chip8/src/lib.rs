mod error;
mod utils;
mod vm;

use wasm_bindgen::prelude::*;

use vm::{Vm, DISPLAY_LEN};

static mut OUTPUT_BUFFER: [u8; 4 * DISPLAY_LEN] = [0; 4 * DISPLAY_LEN];

pub use error::{Error, VmError};
pub type Result<T> = core::result::Result<T, Error>;

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct Emu {
    vm: Vm,
}

#[wasm_bindgen]
impl Emu {
    #[wasm_bindgen(constructor)]
    pub fn new(rom: &[u8]) -> Self {
        Self { vm: Vm::new(rom) }
    }

    #[wasm_bindgen]
    pub fn run(&mut self, cycles: usize) -> Result<bool> {
        let mut shall_halt = false;

        for _ in 0..cycles {
            let res = self.vm.tick();
            shall_halt = match res {
                Ok(x) => x,
                Err(VmError::InvalidOpcode(_)) => true,
                Err(err) => return Err(Error::from(err)),
            };

            self.update_display_buffer();

            if shall_halt {
                break;
            }
        }

        Ok(shall_halt)
    }

    #[wasm_bindgen]
    pub fn display_buffer() -> *const u8 {
        let pointer: *const u8;
        unsafe {
            pointer = OUTPUT_BUFFER.as_ptr();
        }

        pointer
    }

    fn update_display_buffer(&self) {
        for (i, pixel) in self.vm.display.iter().enumerate() {
            let r = if *pixel { 0xff } else { 0x00 };
            let g = if *pixel { 0xff } else { 0x00 };
            let b = if *pixel { 0xff } else { 0x00 };

            unsafe {
                OUTPUT_BUFFER[i * 4 + 0] = r;
                OUTPUT_BUFFER[i * 4 + 1] = g;
                OUTPUT_BUFFER[i * 4 + 2] = b;
                OUTPUT_BUFFER[i * 4 + 3] = 0xff;
            }
        }
    }
}

#[wasm_bindgen(js_name=loadRom)]
pub fn load_rom(rom: &[u8]) -> Result<Emu> {
    utils::set_panic_hook();

    if rom.len() > 4096 - 200 {
        return Err(Error::InvalidRom);
    }

    Ok(Emu::new(rom))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_rom_returns_error_for_invalid_roms() {
        let rom = [0_u8; 4096];
        let res = load_rom(&rom);
        assert_eq!(res, Err(Error::InvalidRom));
    }
}
