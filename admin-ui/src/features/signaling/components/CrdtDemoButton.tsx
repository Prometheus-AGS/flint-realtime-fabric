import { useState } from "react";

type DemoState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "done"; result: string }
  | { status: "error"; message: string };

/**
 * Demonstrates the frf-wasm CRDT binding in-browser.
 *
 * The WASM module is dynamically imported so that the rest of the page
 * loads immediately even if wasm-pack output isn't present yet.
 */
export function CrdtDemoButton(): React.JSX.Element {
  const [state, setState] = useState<DemoState>({ status: "idle" });

  const handleClick = async () => {
    setState({ status: "loading" });
    try {
      // Dynamic import — fails gracefully if WASM not built yet.
      const wasm = await import("frf-wasm");
      if ("default" in wasm && typeof (wasm as { default?: () => Promise<void> }).default === "function") {
        await (wasm as { default: () => Promise<void> }).default();
      }

      // Demo: apply an empty delta to an empty snapshot → should return empty bytes.
      const snapshot = new Uint8Array(0);
      const delta = new Uint8Array(0);
      const merged = wasm.crdt_apply_delta(snapshot, delta);

      setState({
        status: "done",
        result: `crdt_apply_delta([], []) → [${Array.from(merged).join(", ")}] (${merged.byteLength} bytes)`,
      });
    } catch (err) {
      setState({
        status: "error",
        message: err instanceof Error ? err.message : "Unknown error — is frf-wasm built? Run: bash crates/frf-wasm/build_wasm.sh",
      });
    }
  };

  const isLoading = state.status === "loading";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "1rem",
        background: "var(--color-surface-elevated)",
        border: "1px solid var(--color-border)",
        borderRadius: "12px",
        padding: "1.5rem",
      }}
    >
      <div>
        <h2
          style={{ margin: "0 0 0.25rem", fontSize: "1rem", fontWeight: 700, letterSpacing: "-0.01em" }}
        >
          CRDT Demo
        </h2>
        <p style={{ margin: 0, fontSize: "0.8125rem", color: "var(--color-text-muted)" }}>
          Calls{" "}
          <code style={{ fontFamily: "var(--font-mono, monospace)", fontSize: "0.8125rem" }}>
            frf_crdt::apply_delta
          </code>{" "}
          via WebAssembly in-browser.
        </p>
      </div>

      <button
        type="button"
        onClick={handleClick}
        disabled={isLoading}
        aria-busy={isLoading}
        style={{
          alignSelf: "flex-start",
          padding: "0.5rem 1.125rem",
          borderRadius: "8px",
          border: "1px solid var(--color-border)",
          background: isLoading ? "var(--color-surface)" : "var(--color-accent)",
          color: isLoading ? "var(--color-text-muted)" : "white",
          fontWeight: 600,
          fontSize: "0.875rem",
          cursor: isLoading ? "wait" : "pointer",
          transition: "background var(--duration-fast, 150ms), color var(--duration-fast, 150ms)",
        }}
      >
        {isLoading ? "Loading WASM…" : "Run CRDT merge"}
      </button>

      {state.status === "done" && (
        <pre
          data-testid="crdt-result"
          aria-live="polite"
          style={{
            margin: 0,
            padding: "0.75rem 1rem",
            borderRadius: "8px",
            background: "var(--color-surface)",
            border: "1px solid var(--color-border)",
            fontFamily: "var(--font-mono, monospace)",
            fontSize: "0.8125rem",
            overflowX: "auto",
            color: "var(--color-text)",
          }}
        >
          {state.result}
        </pre>
      )}

      {state.status === "error" && (
        <p
          role="alert"
          style={{
            margin: 0,
            padding: "0.625rem 0.875rem",
            borderRadius: "8px",
            background: "var(--status-error-bg, rgba(239,68,68,.1))",
            color: "var(--status-error-dot, #dc2626)",
            fontSize: "0.8125rem",
            fontFamily: "var(--font-mono, monospace)",
          }}
        >
          {state.message}
        </p>
      )}
    </div>
  );
}
