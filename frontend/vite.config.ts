import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";
import devtools from "solid-devtools/vite";
import wasm from "vite-plugin-wasm"

export default defineConfig({
  plugins: [
    wasm(),
    devtools({ autoname: true }),
    solidPlugin(),
    tsconfigPaths(),
  ],
  optimizeDeps: {
    include: ["@codemirror/state", "@codemirror/view"],
  },
  server: {
    port: 3000,
  },
  build: {
    target: "esnext",
  },
  css: {
    preprocessorOptions: {
      sass: {
        silenceDeprecations: [
          "mixed-decls",
        ],
      },
    },
  },
});
