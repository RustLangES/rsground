import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";
import devtools from "solid-devtools/vite";

export default defineConfig({
  plugins: [
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
