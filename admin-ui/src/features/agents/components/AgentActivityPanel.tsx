import { useAgentEventStore } from "../stores/agentEventStore.js";
import type { AgentTransport } from "../stores/agentEventStore.js";
import { useAgentEventStream } from "../hooks/useAgentEventStream.js";
import { useAgentGrpcStream } from "../hooks/useAgentGrpcStream.js";
import { AgentEventRow } from "./AgentEventRow.js";

const STATUS_LABELS = {
  connected: "Connected",
  connecting: "Connecting…",
  disconnected: "Disconnected",
  error: "Error",
} as const;

const STATUS_CLASSES = {
  connected: "status-connected",
  connecting: "status-connecting",
  disconnected: "status-disconnected",
  error: "status-error",
} as const;

const TRANSPORT_LABELS: Record<AgentTransport, string> = {
  websocket: "WS",
  grpc: "gRPC (WASM)",
};

export function AgentActivityPanel() {
  useAgentEventStream();
  useAgentGrpcStream();

  const events = useAgentEventStore((s) => s.events);
  const connectionStatus = useAgentEventStore((s) => s.connectionStatus);
  const lastError = useAgentEventStore((s) => s.lastError);
  const clearEvents = useAgentEventStore((s) => s.clearEvents);
  const transport = useAgentEventStore((s) => s.transport);
  const setTransport = useAgentEventStore((s) => s.setTransport);

  const statusLabel = STATUS_LABELS[connectionStatus];
  const statusClass = STATUS_CLASSES[connectionStatus];

  const toggleTransport = () =>
    setTransport(transport === "websocket" ? "grpc" : "websocket");

  return (
    <section className="agent-activity-panel" aria-labelledby="agent-panel-heading">
      <header className="panel-header">
        <h2 id="agent-panel-heading">Agent Activity</h2>
        <div className="panel-controls">
          <button
            type="button"
            onClick={toggleTransport}
            title={`Switch transport (current: ${TRANSPORT_LABELS[transport]})`}
            aria-label={`Current transport: ${TRANSPORT_LABELS[transport]}. Click to switch.`}
            style={{
              padding: "0.25rem 0.625rem",
              borderRadius: "6px",
              border: "1px solid var(--color-border, #333)",
              background: transport === "grpc" ? "var(--color-accent, #6366f1)" : "transparent",
              color: transport === "grpc" ? "white" : "inherit",
              fontSize: "0.6875rem",
              fontWeight: 600,
              letterSpacing: "0.05em",
              textTransform: "uppercase",
              cursor: "pointer",
            }}
          >
            {TRANSPORT_LABELS[transport]}
          </button>
          <span className={`connection-status ${statusClass}`} role="status" aria-live="polite">
            {statusLabel}
          </span>
          <button
            type="button"
            className="btn-clear"
            onClick={clearEvents}
            disabled={events.length === 0}
            aria-label="Clear agent events"
          >
            Clear
          </button>
        </div>
      </header>

      {lastError && connectionStatus === "error" && (
        <div className="panel-error" role="alert">
          {lastError}
        </div>
      )}

      {events.length === 0 ? (
        <p className="panel-empty">No agent events yet. Events appear here in real time.</p>
      ) : (
        <ul className="agent-event-list" aria-label="Agent events">
          {events.map((event) => (
            <AgentEventRow key={`${event.run_id}-${event.timestamp}-${event.kind}`} event={event} />
          ))}
        </ul>
      )}
    </section>
  );
}
