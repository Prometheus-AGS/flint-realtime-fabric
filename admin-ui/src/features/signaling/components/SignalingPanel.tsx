import { useState } from "react";
import { useSignalingStore } from "../stores/signalingStore.js";
import { useSignalingStream } from "../hooks/useSignalingStream.js";
import type { SignalingStatus } from "../stores/signalingStore.js";

const STATUS_META: Record<SignalingStatus, { label: string; color: string; dot: string }> = {
  idle: { label: "Not connected", color: "var(--status-idle-bg)", dot: "var(--status-idle-dot)" },
  joining: { label: "Connecting…", color: "var(--status-joining-bg)", dot: "var(--status-joining-dot)" },
  joined: { label: "Live", color: "var(--status-joined-bg)", dot: "var(--status-joined-dot)" },
  left: { label: "Disconnected", color: "var(--status-left-bg)", dot: "var(--status-left-dot)" },
  error: { label: "Error", color: "var(--status-error-bg)", dot: "var(--status-error-dot)" },
};

function StatusBadge({ status }: { status: SignalingStatus }) {
  const meta = STATUS_META[status];
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "0.375rem",
        padding: "0.25rem 0.625rem",
        borderRadius: "9999px",
        background: meta.color,
        fontSize: "0.75rem",
        fontWeight: 600,
        letterSpacing: "0.04em",
        textTransform: "uppercase",
      }}
    >
      <span
        aria-hidden="true"
        style={{
          width: "0.5rem",
          height: "0.5rem",
          borderRadius: "50%",
          background: meta.dot,
          flexShrink: 0,
          boxShadow: status === "joined" ? `0 0 0 3px color-mix(in srgb, ${meta.dot} 30%, transparent)` : "none",
        }}
      />
      {meta.label}
    </span>
  );
}

function ParticipantList({ participants }: { participants: string[] }) {
  if (participants.length === 0) {
    return (
      <p style={{ color: "var(--color-text-muted)", fontSize: "0.8125rem", margin: 0 }}>
        No participants yet
      </p>
    );
  }
  return (
    <ul style={{ listStyle: "none", margin: 0, padding: 0, display: "flex", flexDirection: "column", gap: "0.375rem" }}>
      {participants.map((p) => (
        <li
          key={p}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "0.5rem",
            fontSize: "0.8125rem",
            fontFamily: "var(--font-mono, monospace)",
            color: "var(--color-text-secondary)",
          }}
        >
          <span
            aria-hidden="true"
            style={{
              width: "1.5rem",
              height: "1.5rem",
              borderRadius: "4px",
              background: `hsl(${Math.abs(hashCode(p)) % 360} 60% 65%)`,
              flexShrink: 0,
            }}
          />
          {p.slice(0, 8)}…
        </li>
      ))}
    </ul>
  );
}

function hashCode(s: string): number {
  return s.split("").reduce((acc, c) => (acc * 31 + c.charCodeAt(0)) | 0, 0);
}

export function SignalingPanel(): React.JSX.Element {
  const { status, roomId, participants, lastError, joinRoom, leaveRoom } = useSignalingStore();
  const [inputRoom, setInputRoom] = useState("");

  useSignalingStream();

  const handleJoin = () => {
    const trimmed = inputRoom.trim();
    if (trimmed) joinRoom(trimmed);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") handleJoin();
  };

  return (
    <section
      aria-labelledby="signaling-panel-heading"
      style={{
        display: "grid",
        gridTemplateRows: "auto 1fr",
        gap: "1.25rem",
        background: "var(--color-surface-elevated)",
        border: "1px solid var(--color-border)",
        borderRadius: "12px",
        padding: "1.5rem",
        position: "relative",
        overflow: "hidden",
      }}
    >
      {/* Accent stripe */}
      <span
        aria-hidden="true"
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          height: "3px",
          background: "var(--color-accent)",
          borderRadius: "12px 12px 0 0",
        }}
      />

      {/* Header */}
      <header style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "1rem" }}>
        <div>
          <h2
            id="signaling-panel-heading"
            style={{ margin: 0, fontSize: "1rem", fontWeight: 700, letterSpacing: "-0.01em" }}
          >
            WebRTC Signaling
          </h2>
          {roomId && (
            <p style={{ margin: "0.25rem 0 0", fontSize: "0.8125rem", color: "var(--color-text-muted)", fontFamily: "var(--font-mono, monospace)" }}>
              room: {roomId}
            </p>
          )}
        </div>
        <StatusBadge status={status} />
      </header>

      {/* Body */}
      <div style={{ display: "flex", flexDirection: "column", gap: "1.25rem" }}>
        {/* Join / leave controls */}
        {status === "idle" || status === "left" || status === "error" ? (
          <div style={{ display: "flex", gap: "0.5rem" }}>
            <input
              type="text"
              value={inputRoom}
              onChange={(e) => setInputRoom(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Room ID"
              aria-label="Room ID"
              style={{
                flex: 1,
                padding: "0.5rem 0.75rem",
                borderRadius: "8px",
                border: "1px solid var(--color-border)",
                background: "var(--color-surface)",
                color: "var(--color-text)",
                fontSize: "0.875rem",
                outline: "none",
                transition: "border-color var(--duration-fast, 150ms)",
              }}
            />
            <button
              type="button"
              onClick={handleJoin}
              disabled={!inputRoom.trim()}
              style={{
                padding: "0.5rem 1rem",
                borderRadius: "8px",
                border: "none",
                background: "var(--color-accent)",
                color: "white",
                fontWeight: 600,
                fontSize: "0.875rem",
                cursor: "pointer",
                transition: "opacity var(--duration-fast, 150ms)",
                opacity: inputRoom.trim() ? 1 : 0.4,
              }}
            >
              Join
            </button>
          </div>
        ) : (
          <button
            type="button"
            onClick={leaveRoom}
            style={{
              alignSelf: "flex-start",
              padding: "0.5rem 1rem",
              borderRadius: "8px",
              border: "1px solid var(--color-border)",
              background: "transparent",
              color: "var(--color-text)",
              fontWeight: 600,
              fontSize: "0.875rem",
              cursor: "pointer",
            }}
          >
            Leave
          </button>
        )}

        {/* Error message */}
        {lastError && (
          <p
            role="alert"
            style={{
              margin: 0,
              padding: "0.625rem 0.875rem",
              borderRadius: "8px",
              background: "var(--status-error-bg, rgba(239,68,68,.1))",
              color: "var(--status-error-dot, #dc2626)",
              fontSize: "0.8125rem",
            }}
          >
            {lastError}
          </p>
        )}

        {/* Participants */}
        <div>
          <h3
            style={{
              margin: "0 0 0.625rem",
              fontSize: "0.75rem",
              fontWeight: 600,
              textTransform: "uppercase",
              letterSpacing: "0.06em",
              color: "var(--color-text-muted)",
            }}
          >
            Participants ({participants.length})
          </h3>
          <ParticipantList participants={participants} />
        </div>
      </div>
    </section>
  );
}
