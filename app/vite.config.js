import wasm from "vite-plugin-wasm";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [wasm()],
  assetsInclude: ["**/*.svg", "**/*.ch8"],
  build: {
    target: "esnext",
  },
});
