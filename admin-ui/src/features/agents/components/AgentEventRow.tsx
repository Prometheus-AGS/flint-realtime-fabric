import type { AgentEvent } from "../types/agent.js";
import { ContentBlockPreview } from "./ContentBlockPreview.js";

interface AgentEventRowProps {
  event: AgentEvent;
}

const PROTOCOL_LABELS: Record<string, string> = {
  ag_ui: "AG-UI",
  a2a: "A2A",
  a2ui: "A2UI",
};

const KIND_BADGES: Record<string, string> = {
  run_start: "badge-start",
  run_end: "badge-end",
  text_delta: "badge-text",
  tool_call: "badge-tool",
  tool_result: "badge-tool",
  state_snapshot: "badge-state",
  error: "badge-error",
};

function formatTimestamp(ts: string): string {
  try {
    return new Date(ts).toLocaleTimeString(undefined, { hour12: false });
  } catch {
    return ts;
  }
}

export function AgentEventRow({ event }: AgentEventRowProps) {
  const protocolLabel = PROTOCOL_LABELS[event.protocol] ?? event.protocol;
  const kindClass = KIND_BADGES[event.kind] ?? "badge-default";

  return (
    <li className="agent-event-row">
      <span className="event-time">{formatTimestamp(event.timestamp)}</span>
      <span className={`event-badge ${kindClass}`}>{event.kind}</span>
      <span className="event-protocol">{protocolLabel}</span>
      <span className="event-agent" title={event.agent_id}>
        {event.agent_id.slice(0, 8)}
      </span>
      <span className="event-content">
        <ContentBlockPreview block={event.content} />
      </span>
    </li>
  );
}
