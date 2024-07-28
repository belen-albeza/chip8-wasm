import wasmInit, { loadRom, Emu } from "chip8";

const ROMS = [
  { name: "poker.ch8", url: "/roms/poker.ch8" },
  { name: "wait_for_key.ch8", url: "/roms/wait_for_key.ch8" },
];
const DISPLAY_LEN = 64 * 32;
const THEMES = [
  {
    name: "Noire Truth",
    off: "#1e1c32",
    on: "#c6baac",
  },
  {
    name: "1-bit Monitor Glow",
    off: "#222323",
    on: "#f0f6f0",
  },
  {
    name: "Paperback-2",
    off: "#382b26",
    on: "#b8c2b9",
  },
  {
    name: "Gato Roboto Goop",
    off: "#210009",
    on: "#00ffae",
  },
  {
    name: "Y's Postapocalyptic Sunset",
    off: "#1d0f44",
    on: "#f44e38",
  },
];

let animationFrameRequestId: number;
let keyDownController = new AbortController();
let keyUpController = new AbortController();

const config = {
  cyclesPerFrame: 12,
  theme: THEMES[0],
};

main();

async function main() {
  setupRomSelector(ROMS);
  startEmulatorWithRom(ROMS[0].url);
}

async function startEmulatorWithRom(romUrl: string) {
  if (animationFrameRequestId) {
    cancelAnimationFrame(animationFrameRequestId);
    keyDownController.abort();
    keyUpController.abort();

    keyDownController = new AbortController();
    keyUpController = new AbortController();
  }

  const wasm = await wasmInit();
  const emu = await loadRomInEmu(romUrl);

  setupConfigPanel(emu);
  emu.setTheme(config.theme.off, config.theme.on);

  document.addEventListener(
    "keydown",
    (event) => {
      emu.updateKeyState(event.code, true);
    },
    { signal: keyDownController.signal }
  );

  document.addEventListener(
    "keyup",
    (event) => {
      emu.updateKeyState(event.code, false);
    },
    { signal: keyUpController.signal }
  );

  const sharedBuffer = new Uint8Array(wasm.memory.buffer);
  const canvas = document.querySelector<HTMLCanvasElement>("#chip8-canvas");
  const ctx = canvas?.getContext("2d");
  if (!ctx || !canvas) {
    throw new Error("Valid canvas not found");
  }
  const imageData = ctx.createImageData(canvas.width, canvas.height);

  const updateCanvas = () => {
    let shallHalt = emu.run(config.cyclesPerFrame);

    const outputPointer = emu.displayBuffer();
    const bufferData = sharedBuffer.slice(
      outputPointer,
      outputPointer + 4 * DISPLAY_LEN
    );
    imageData.data.set(bufferData);
    ctx.putImageData(imageData, 0, 0);

    if (shallHalt) {
      console.debug("Chip-8 VM halted");
    } else {
      animationFrameRequestId = requestAnimationFrame(updateCanvas);
    }
  };

  updateCanvas();
}

function setupRomSelector(roms: { name: string; url: string }[]) {
  const selectEl = document.querySelector<HTMLSelectElement>(
    "#chip8-rom-selector"
  );
  for (const { name, url } of roms) {
    const option = new Option();
    option.value = url;
    option.innerHTML = name;
    option.defaultSelected = name === roms[0]?.name;

    selectEl?.appendChild(option);
  }

  selectEl?.addEventListener("change", () => {
    let url = selectEl.value;
    startEmulatorWithRom(url);
  });
}

function setupConfigPanel(emu: Emu) {
  const cyclesInput = document.querySelector(
    "#chip8-ipf-selector"
  ) as HTMLInputElement;

  cyclesInput.value = config.cyclesPerFrame.toString();
  cyclesInput.addEventListener("change", () => {
    const cycles = parseInt(cyclesInput.value);
    config.cyclesPerFrame = cycles;
    updateCyclesPerSecond();
  });

  const updateCyclesPerSecond = () => {
    const cyclesPerSecond = document.querySelector(
      "#chip8-config-ips"
    ) as HTMLElement;
    const count = config.cyclesPerFrame * 60;
    cyclesPerSecond.textContent = count.toString();
  };

  updateCyclesPerSecond();

  const themeSelect = document.querySelector<HTMLSelectElement>(
    "#chip8-config-theme-selector"
  );
  for (const [index, { name }] of THEMES.entries()) {
    const option = new Option();
    option.value = index.toString();
    option.innerText = name;
    themeSelect?.appendChild(option);
  }
  themeSelect?.addEventListener("change", () => {
    const idx = parseInt(themeSelect.value);
    config.theme = THEMES[idx];
    emu.setTheme(config.theme.off, config.theme.on);
  });
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
