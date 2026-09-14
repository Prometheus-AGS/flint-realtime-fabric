# p7-c005 — Admin-UI WebRTC + WASM Integration

## Summary

Integrate the `frf-wasm` SDK into admin-ui: add a build hook, wire `AgentStream`
into the agent activity panel, add `WebRtcPanel` component, and remove the
hand-authored WASM type stub.

## Changes

### `admin-ui/package.json`
- Add `"prepare": "test -n \"$CI\" || bash ../crates/frf-wasm/build_wasm.sh"` under `scripts`
  (CI=true skips WASM build; developers get it on `pnpm install`)
- Add `"frf-wasm": "file:../sdks/ts/frf-wasm"` under `dependencies`

### `admin-ui/src/types/frf-wasm.d.ts`
- Delete — replaced by wasm-pack generated `sdks/ts/frf-wasm/frf_wasm.d.ts`

### `admin-ui/src/features/agents/hooks/useAgentEventStream.ts`
- Attempt to import `AgentStream` from `frf-wasm` dynamically
- If WASM available and `accessToken` is set: use `AgentStream.open(token, onEvent)` to populate `agentEventStore`
- If WASM unavailable (dev without build): fall back to existing WS implementation
- Add `import.meta.env.DEV` guard for the fallback path

### `admin-ui/src/features/signaling/components/WebRtcPanel.tsx` (NEW)
```tsx
// Shows peer connection status, ICE state, and offer/answer controls.
// Uses useWebRtc hook; renders in SignalingPanel.
export function WebRtcPanel(): JSX.Element
```

### `admin-ui/src/features/signaling/hooks/useWebRtc.ts` (NEW)
```typescript
// Manages RTCPeerConnection lifecycle.
// Connects via SpineSignalService (Connect-ES bidi streaming) for ICE/SDP exchange.
export function useWebRtc(sessionId: string): {
  state: RTCPeerConnectionState;
  iceState: RTCIceGatheringState;
  startCall: () => Promise<void>;
  endCall: () => void;
}
```

### `admin-ui/src/features/signaling/components/SignalingPanel.tsx`
- Import and render `WebRtcPanel` below existing signaling status display

### `admin-ui/src/features/agents/components/AgentActivityPanel.tsx` (or existing panel)
- Add connection status badge: "WASM" when using AgentStream, "WS" when falling back
- No breaking changes to existing panel structure

## Quality Gates

- `fnm exec --using=24 pnpm typecheck` — 0 errors
- `fnm exec --using=24 pnpm build` — 0 errors
- No `any` types
- No hardcoded tokens
