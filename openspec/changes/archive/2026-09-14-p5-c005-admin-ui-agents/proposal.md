# p5-c005 — Admin UI agents feature

## Summary

Add `admin-ui/src/features/agents/` with a live agent activity panel that
consumes the `/ws/v1/agents` WebSocket stream and renders `AgentEvent` rows
in real time.

## Motivation

Phase 5's exit criterion requires a browser consumer. The agents feature panel
is that consumer — it also serves as the operational monitor for AG-UI, A2A,
and A2UI protocol traffic.

## Design

### Directory structure

```
admin-ui/src/features/agents/
├── components/
│   ├── AgentActivityPanel.tsx   # main panel component
│   ├── AgentEventRow.tsx        # single event row (kind badge + content preview)
│   └── ContentBlockPreview.tsx  # renders typed ContentBlock
├── hooks/
│   └── useAgentEventStream.ts   # WS subscription hook
├── stores/
│   └── agentEventStore.ts       # Zustand ring buffer (max 200 events)
├── services/
│   └── agentWebSocket.ts        # raw WS connection management
└── types/
    └── agent.ts                 # TypeScript mirrors of AgentEvent, ContentBlock
```

### Types (TypeScript mirrors of proto)

```ts
// features/agents/types/agent.ts

export type AgentProtocol = "ag_ui" | "a2a" | "a2ui";

export type AgentEventKind =
  | "run_start" | "run_end" | "text_delta"
  | "tool_call" | "tool_result" | "state_snapshot" | "error";

export interface AgentEvent {
  agent_id: string;
  tenant_id: string;
  session_id: string;
  protocol: AgentProtocol;
  kind: AgentEventKind;
  run_id: string;
  content: ContentBlock;
  timestamp: string; // ISO-8601
}

export type ContentBlock =
  | { type: "text_delta"; delta: string }
  | { type: "tool_call"; tool_name: string; input: unknown }
  | { type: "tool_result"; tool_name: string; output: unknown; is_error: boolean }
  | { type: "state_snapshot"; state: unknown }
  | { type: "run_start"; model?: string }
  | { type: "run_end"; stop_reason?: string }
  | { type: "error"; message: string; code?: string }
  | { type: string; [key: string]: unknown }; // unknown/future variant
```

### Panel component

```tsx
// AgentActivityPanel.tsx
export function AgentActivityPanel() {
  const events = useAgentEventStore(s => s.events);
  useAgentEventStream(); // mounts WS on component mount

  return (
    <section aria-labelledby="agent-panel-heading">
      <h2 id="agent-panel-heading">Agent Activity</h2>
      <ol aria-live="polite" aria-label="Agent event stream">
        {events.map(e => <AgentEventRow key={`${e.run_id}-${e.timestamp}`} event={e} />)}
      </ol>
    </section>
  );
}
```

### Design direction

- Dark luxury / editorial style matching Phase 4 signaling panel
- `AgentEventKind` rendered as a color-coded badge (run_start=blue, text_delta=green, error=red)
- `ContentBlock` content collapsed by default; expandable on click
- Ring buffer: max 200 events, oldest evicted when full (Zustand store)
- Accessible: `aria-live="polite"` on the event list

## Files Changed

- `admin-ui/src/features/agents/types/agent.ts` — NEW
- `admin-ui/src/features/agents/services/agentWebSocket.ts` — NEW
- `admin-ui/src/features/agents/stores/agentEventStore.ts` — NEW (Zustand)
- `admin-ui/src/features/agents/hooks/useAgentEventStream.ts` — NEW
- `admin-ui/src/features/agents/components/AgentEventRow.tsx` — NEW
- `admin-ui/src/features/agents/components/ContentBlockPreview.tsx` — NEW
- `admin-ui/src/features/agents/components/AgentActivityPanel.tsx` — NEW
- `admin-ui/src/App.tsx` — add Agents tab/route

## Acceptance Criteria

- [ ] `pnpm typecheck` passes (no `any` types)
- [ ] Panel renders with no gateway (shows empty state, not crash)
- [ ] WS connection reconnects on close (exponential backoff in `agentWebSocket.ts`)
- [ ] Ring buffer evicts correctly at 200 events (unit test)
- [ ] Semantic HTML: `<section>`, `<ol>`, `aria-live`
- [ ] `data-testid="agent-activity-panel"` on the section root for E2E
- [ ] `data-testid="agent-event-row"` on each row for E2E assertions
