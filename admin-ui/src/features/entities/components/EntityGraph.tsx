import { useEntitySubscription } from "../hooks/useEntitySubscription.js";
import type { EntityEvent } from "../types.js";

interface EntityGraphProps {
  channelId: string;
  consumerId: string;
  entityType?: string;
}

function EventRow({ event }: { event: EntityEvent }): React.JSX.Element {
  return (
    <tr style={{ borderBottom: "1px solid var(--color-border)" }}>
      <td style={{ padding: "0.5rem 0.75rem", fontFamily: "monospace", fontSize: "0.8rem" }}>
        {event.entityType}
      </td>
      <td style={{ padding: "0.5rem 0.75rem", fontFamily: "monospace", fontSize: "0.8rem" }}>
        {event.entityId}
      </td>
      <td style={{ padding: "0.5rem 0.75rem", fontSize: "0.8rem" }}>
        {event.tenantId}
      </td>
      <td style={{ padding: "0.5rem 0.75rem", fontSize: "0.8rem", color: "var(--color-muted)" }}>
        {String(event.offset)}
      </td>
    </tr>
  );
}

export function EntityGraph({ channelId, consumerId, entityType }: EntityGraphProps): React.JSX.Element {
  const { events, connected, error } = useEntitySubscription({
    channelId,
    consumerId,
    ...(entityType !== undefined ? { entityType } : {}),
  });

  return (
    <section aria-label="Entity event stream">
      <header style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "1rem" }}>
        <h2 style={{ margin: 0, fontSize: "1rem", fontWeight: 600 }}>Entity Events</h2>
        <span
          aria-label={connected ? "Connected" : "Disconnected"}
          style={{
            display: "inline-block",
            width: "0.5rem",
            height: "0.5rem",
            borderRadius: "50%",
            background: connected ? "#22c55e" : "#ef4444",
          }}
        />
        <span style={{ fontSize: "0.75rem", color: "var(--color-muted)" }}>
          {connected ? "live" : "disconnected"}
        </span>
      </header>

      {error !== null && (
        <p role="alert" style={{ color: "#ef4444", fontSize: "0.875rem", marginBottom: "0.75rem" }}>
          {error}
        </p>
      )}

      {events.length === 0 ? (
        <p style={{ color: "var(--color-muted)", fontSize: "0.875rem" }}>
          Waiting for entity change events…
        </p>
      ) : (
        <div style={{ overflowX: "auto" }}>
          <table
            style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.875rem" }}
            aria-label="Entity events table"
          >
            <thead>
              <tr style={{ textAlign: "left", borderBottom: "2px solid var(--color-border)" }}>
                <th style={{ padding: "0.5rem 0.75rem", fontWeight: 600 }}>Type</th>
                <th style={{ padding: "0.5rem 0.75rem", fontWeight: 600 }}>ID</th>
                <th style={{ padding: "0.5rem 0.75rem", fontWeight: 600 }}>Tenant</th>
                <th style={{ padding: "0.5rem 0.75rem", fontWeight: 600 }}>Offset</th>
              </tr>
            </thead>
            <tbody>
              {events.map((ev) => (
                <EventRow key={`${ev.channelId}-${String(ev.offset)}`} event={ev} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
