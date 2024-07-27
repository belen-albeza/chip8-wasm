import wasmInit, { loadRom, Emu } from "chip8";

const ROMS = [{ name: "poker.ch8", url: "/roms/poker.ch8" }];
const DISPLAY_LEN = 64 * 32;

main();

async function main() {
  const wasm = await wasmInit();

  setupRomSelector(ROMS);

  const emu = await loadRomInEmu(ROMS[0].url);

  const sharedBuffer = new Uint8Array(wasm.memory.buffer);
  const canvas = document.querySelector<HTMLCanvasElement>("#chip8-canvas");
  const ctx = canvas?.getContext("2d");
  if (!ctx || !canvas) {
    throw new Error("Valid canvas not found");
  }
  const imageData = ctx.createImageData(canvas.width, canvas.height);

  const updateCanvas = () => {
    let shallHalt = emu.run(16);

    const outputPointer = Emu.display_buffer();
    const bufferData = sharedBuffer.slice(
      outputPointer,
      outputPointer + 4 * DISPLAY_LEN
    );
    imageData.data.set(bufferData);
    ctx.putImageData(imageData, 0, 0);

    if (shallHalt) {
      console.debug("Chip-8 VM halted");
    } else {
      requestAnimationFrame(updateCanvas);
    }
  };

  updateCanvas();
}

function setupRomSelector(roms: { name: string; url: string }[]) {
  const selectEl = document.querySelector("#chip8-rom-selector");
  for (const { name, url } of roms) {
    const option = new Option();
    option.value = url;
    option.innerHTML = name;
    option.defaultSelected = name === roms[0]?.name;

    selectEl?.appendChild(option);
  }
}

async function fetchRom(url: string) {
  try {
    const response = await fetch(url);
    return response;
  } catch (err) {
    console.error(err);
  }
}

async function loadRomInEmu(url: string) {
  const file = await fetchRom(url);
  if (!file) {
    throw new Error(`Could not load rom: ${url}`);
  }

  const buffer = await file.arrayBuffer();
  const rom = new Uint8Array(buffer);

  const emu = loadRom(rom);
  return emu;
}
