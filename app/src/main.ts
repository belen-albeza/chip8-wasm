import { loadRom } from "chip8";

const ROMS = [{ name: "poker.ch8", url: "/roms/poker.ch8" }];

main();

function main() {
  setupRomSelector(ROMS);
  runRom(ROMS[0].url).then((vm) => {
    vm.run();
  });
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

async function runRom(url: string) {
  const file = await fetchRom(url);
  if (!file) {
    throw new Error(`Could not load rom: ${url}`);
  }

  const buffer = await file.arrayBuffer();
  const rom = new Uint8Array(buffer);

  const vm = loadRom(rom);
  return vm;
}
