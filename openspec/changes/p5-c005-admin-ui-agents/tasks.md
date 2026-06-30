# Tasks — p5-c005-admin-ui-agents

- [ ] Create `admin-ui/src/features/agents/types/agent.ts` with `AgentEvent`, `ContentBlock`, `AgentEventKind`, `AgentProtocol` TypeScript types
- [ ] Create `admin-ui/src/features/agents/services/agentWebSocket.ts` with WS connect/reconnect logic (exponential backoff)
- [ ] Create `admin-ui/src/features/agents/stores/agentEventStore.ts` (Zustand, max 200 event ring buffer)
- [ ] Create `admin-ui/src/features/agents/hooks/useAgentEventStream.ts` connecting WS service to Zustand store
- [ ] Create `admin-ui/src/features/agents/components/ContentBlockPreview.tsx` with collapsed/expanded toggle
- [ ] Create `admin-ui/src/features/agents/components/AgentEventRow.tsx` with kind badge + content preview
- [ ] Create `admin-ui/src/features/agents/components/AgentActivityPanel.tsx` with aria-live list
- [ ] Add `data-testid` attributes: `agent-activity-panel`, `agent-event-row`
- [ ] Wire `AgentActivityPanel` into `admin-ui/src/App.tsx` (new tab or route)
- [ ] Unit test: ring buffer evicts oldest event when count exceeds 200
- [ ] `pnpm typecheck` clean (no `any` types, no TypeScript errors)
- [ ] `pnpm build` clean
