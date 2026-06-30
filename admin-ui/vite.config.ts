import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import type { Plugin } from "vite";

function frfWasmStubPlugin(): Plugin {
  const require = createRequire(import.meta.url);
  let wasmAvailable = false;
  try {
    require.resolve("frf-wasm");
    wasmAvailable = true;
  } catch {
    wasmAvailable = false;
  }

  if (wasmAvailable) return { name: "frf-wasm-stub" };

  const STUB_ID = "frf-wasm";
  const RESOLVED_ID = "\0frf-wasm-stub";

  return {
    name: "frf-wasm-stub",
    resolveId(id: string) {
      if (id === STUB_ID) return RESOLVED_ID;
      return null;
    },
    load(id: string) {
      if (id === RESOLVED_ID) {
        return `
export function crdt_apply_delta(snapshot, delta) {
  throw new Error("frf-wasm not built — run: bash crates/frf-wasm/build_wasm.sh");
}
export default async function init() {
  throw new Error("frf-wasm not built — run: bash crates/frf-wasm/build_wasm.sh");
}
`;
      }
      return null;
    },
  };
}

export default defineConfig({
  plugins: [frfWasmStubPlugin(), react()],
  build: {
    target: "es2022",
    outDir: "dist",
  },
  server: {
    port: 5173,
  },
});
