import { SignalingPanel } from "../components/SignalingPanel.js";
import { CrdtDemoButton } from "../components/CrdtDemoButton.js";

export function SignalingDemoPage(): React.JSX.Element {
  return (
    <main
      style={{
        padding: "2rem",
        maxWidth: "960px",
        margin: "0 auto",
        display: "grid",
        gap: "1.5rem",
      }}
    >
      <header>
        <h1
          style={{
            fontSize: "1.5rem",
            fontWeight: 700,
            letterSpacing: "-0.02em",
            margin: "0 0 0.25rem",
          }}
        >
          Signaling Demo
        </h1>
        <p style={{ margin: 0, fontSize: "0.9375rem", color: "var(--color-text-muted)" }}>
          WebRTC signaling relay + in-browser CRDT via WebAssembly.
        </p>
      </header>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0,1.5fr) minmax(0,1fr)",
          gap: "1.25rem",
          alignItems: "start",
        }}
      >
        <SignalingPanel />
        <CrdtDemoButton />
      </div>
    </main>
  );
}
