/**
 * Type stubs for the frf-wasm wasm-bindgen output.
 *
 * The real module is produced by `bash crates/frf-wasm/build_wasm.sh` and
 * placed at `sdks/ts/frf-wasm/`. Until built, dynamic imports of "frf-wasm"
 * will fail at runtime and be caught by `CrdtDemoButton`'s error handler.
 */
declare module "frf-wasm" {
  /** Apply a Loro CRDT delta to a snapshot and return the merged snapshot. */
  export function crdt_apply_delta(snapshot: Uint8Array, delta: Uint8Array): Uint8Array;

  /**
   * Initialise the WASM module.  Must be awaited before calling other exports
   * when using the `web` wasm-pack target.
   */
  export default function init(input?: RequestInfo | URL | BufferSource | WebAssembly.Module): Promise<void>;
}
