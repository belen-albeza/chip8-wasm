mod error;
mod utils;
mod vm;

use regex::RegexBuilder;
use wasm_bindgen::prelude::*;

use vm::{Vm, DISPLAY_LEN};

static mut OUTPUT_BUFFER: [u8; 4 * DISPLAY_LEN] = [0; 4 * DISPLAY_LEN];

pub use error::{Error, VmError};
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Theme {
    off_color: (u8, u8, u8),
    on_color: (u8, u8, u8),
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            off_color: (0x00, 0x00, 0x00),
            on_color: (0xff, 0xff, 0xff),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct Emu {
    vm: Vm<fn() -> u8>,
    theme: Theme,
}

#[wasm_bindgen]
impl Emu {
    #[wasm_bindgen(constructor)]
    pub fn new(rom: &[u8]) -> Self {
        Self {
            vm: Vm::new(rom, || rand::random()),
            theme: Theme::default(),
        }
    }

    #[wasm_bindgen]
    pub fn run(&mut self, cycles: usize) -> Result<bool> {
        let mut shall_halt = false;

        for _ in 0..cycles {
            let res = self.vm.tick();
            shall_halt = match res {
                Ok(_) => false,
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

    #[wasm_bindgen(js_name=setTheme)]
    pub fn set_theme(&mut self, off_color: &str, on_color: &str) -> Result<()> {
        self.theme = Theme {
            off_color: parse_hex_color(off_color)?,
            on_color: parse_hex_color(on_color)?,
        };

        Ok(())
    }

    #[wasm_bindgen(js_name=displayBuffer)]
    pub fn display_buffer(&self) -> *const u8 {
        let pointer: *const u8;
        unsafe {
            pointer = OUTPUT_BUFFER.as_ptr();
        }

        pointer
    }

    #[wasm_bindgen(js_name=updateKeyState)]
    pub fn update_key_state(&mut self, key_code: &str, value: bool) -> Result<()> {
        let mapped_key = match key_code {
            "Digit1" => Some(0x1),
            "Digit2" => Some(0x2),
            "Digit3" => Some(0x3),
            "Digit4" => Some(0xc),
            "KeyQ" => Some(0x4),
            "KeyW" => Some(0x5),
            "KeyE" => Some(0x6),
            "KeyR" => Some(0xd),
            "KeyA" => Some(0x7),
            "KeyS" => Some(0x8),
            "KeyD" => Some(0x9),
            "KeyF" => Some(0xe),
            "KeyZ" => Some(0xa),
            "KeyX" => Some(0x0),
            "KeyC" => Some(0xb),
            "KeyV" => Some(0xf),
            _ => None,
        };

        if let Some(key) = mapped_key {
            self.vm.set_key(key, value)?;
        }

        Ok(())
    }

    fn update_display_buffer(&self) {
        for (i, pixel) in self.vm.display.iter().enumerate() {
            let (r, g, b) = if *pixel {
                self.theme.on_color
            } else {
                self.theme.off_color
            };

            unsafe {
                OUTPUT_BUFFER[i * 4 + 0] = r;
                OUTPUT_BUFFER[i * 4 + 1] = g;
                OUTPUT_BUFFER[i * 4 + 2] = b;
                OUTPUT_BUFFER[i * 4 + 3] = 0xff;
            }
        }
    }
}

fn parse_hex_color(hex: &str) -> Result<(u8, u8, u8)> {
    let re = RegexBuilder::new(r"#(?<r>[0-9a-f]{2})(?<g>[0-9a-f]{2})(?<b>[0-9a-f]{2})")
        .case_insensitive(true)
        .build()
        .unwrap();
    if let Some(caps) = re.captures(hex) {
        let r = u8::from_str_radix(&caps["r"], 16).map_err(|_| Error::InvalidTheme)?;
        let g = u8::from_str_radix(&caps["g"], 16).map_err(|_| Error::InvalidTheme)?;
        let b = u8::from_str_radix(&caps["b"], 16).map_err(|_| Error::InvalidTheme)?;
        Ok((r, g, b))
    } else {
        Err(Error::InvalidTheme)
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

    #[test]
    fn parses_color_from_hex() {
        assert_eq!(parse_hex_color("#faBAda"), Ok((0xfa, 0xba, 0xda)));
    }
}
