/**
 * Connect-ES gRPC streaming transport for `AgentService.RunAgent`.
 *
 * Attempts to load the `frf_wasm` WASM module and uses its `AgentStream`
 * class to open a Connect-ES server-streaming RPC to the gateway.  Falls
 * back gracefully to a no-op when the WASM module is not present (e.g. in
 * development before `wasm-pack build` has run).
 *
 * Wire this service into `useAgentGrpcStream` — do not call it directly from
 * component code.
 */

import { useAgentEventStore } from "../stores/agentEventStore.js";
import type { AgentEvent } from "../types/agent.js";

const GATEWAY_URL = import.meta.env["VITE_GATEWAY_URL"] ?? "http://localhost:4000";

type WasmModule = {
  AgentStream: new (gatewayUrl: string, token: string) => {
    open: (requestJson: string, onEvent: (frame: Uint8Array) => void) => Promise<void>;
  };
};

let wasmModule: WasmModule | null = null;
let loadAttempted = false;

async function loadWasm(): Promise<WasmModule | null> {
  if (loadAttempted) return wasmModule;
  loadAttempted = true;
  try {
    // Dynamic import — resolved by Vite via the "frf-wasm" workspace package.
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore — module alias resolved by Vite workspace dep
    const mod = await import("frf-wasm");
    await (mod as { default?: () => Promise<void> }).default?.();
    wasmModule = mod as unknown as WasmModule;
    return wasmModule;
  } catch {
    // WASM not available — fall back silently.
    return null;
  }
}

let activeStream: AbortController | null = null;

/**
 * Open a Connect-ES bidi stream via the WASM `AgentStream` class.
 *
 * @param token  JWT Bearer token
 * @param agentId  Agent to run
 * @param sessionId  Caller's session identifier
 */
export async function openGrpcAgentStream(
  token: string,
  agentId: string,
  sessionId: string,
): Promise<void> {
  closeGrpcAgentStream();

  const wasm = await loadWasm();
  if (!wasm) return;

  const { setConnectionStatus, setError, addEvent } = useAgentEventStore.getState();
  setConnectionStatus("connecting");

  const ctrl = new AbortController();
  activeStream = ctrl;

  const stream = new wasm.AgentStream(GATEWAY_URL, token);

  const requestJson = JSON.stringify({
    start: {
      agent_id: agentId,
      session_id: sessionId,
      tenant_id: import.meta.env["VITE_TENANT_ID"] ?? "00000000-0000-0000-0000-000000000001",
      protocol: 1,
      input: {},
    },
  });

  setConnectionStatus("connected");

  try {
    await stream.open(requestJson, (frame: Uint8Array) => {
      if (ctrl.signal.aborted) return;
      try {
        const text = new TextDecoder().decode(frame);
        const agentEvent = JSON.parse(text) as AgentEvent;
        addEvent(agentEvent);
      } catch {
        // Non-decodable frame — ignore.
      }
    });
  } catch (err) {
    if (!ctrl.signal.aborted) {
      setError(err instanceof Error ? err.message : "gRPC stream error");
    }
  } finally {
    if (!ctrl.signal.aborted) {
      setConnectionStatus("disconnected");
    }
  }
}

export function closeGrpcAgentStream(): void {
  if (activeStream) {
    activeStream.abort();
    activeStream = null;
  }
}
