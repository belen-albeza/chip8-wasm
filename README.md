# chip8-wasm

CHIP-8 emulator in WebAssembly

## Build and run

Requirements:

- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) CLI
- [Bun](https://bun.sh/)

1. Build the `chip8` wasm module in Rust:

```
cd chip8
wasm-pack build --target web --out-dir ../app/vendor/chip8
```
