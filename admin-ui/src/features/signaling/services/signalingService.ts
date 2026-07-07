import { useSignalingStore } from "../stores/signalingStore.js";
import type { SignalFrame } from "../stores/signalingStore.js";
import { useAuthStore } from "../../auth/stores/authStore.js";

// The /ws/v1/signal endpoint is an Axum WebSocket route on the gateway's HTTP
// port (BIND_ADDR, default 8080) — distinct from the gRPC-web port used by
// spineClient. Direct dev → :8080; the compose stack maps it to host 28080, so
// set VITE_GATEWAY_WS_URL=http://localhost:28080 against compose.
const GATEWAY_WS_URL = (import.meta.env["VITE_GATEWAY_WS_URL"] ?? "http://localhost:8080")
  .replace(/^http/, "ws");

let activeSocket: WebSocket | null = null;

/**
 * Open a WebSocket to the gateway's signaling endpoint and pump
 * inbound frames into the Zustand store.
 *
 * Only one active stream per tab — calling this while already joined
 * closes the previous stream first.
 */
export async function openSignalStream(roomId: string, tenantId: string): Promise<void> {
  closeSignalStream();

  const { setStatus, setError, onSignalFrame } = useSignalingStore.getState();

  try {
    // Browser WebSocket cannot set an Authorization header, so the JWT (when
    // present) is passed as a query param — the gateway verifies it there.
    const token = useAuthStore.getState().accessToken;
    const tokenParam = token ? `&token=${encodeURIComponent(token)}` : "";
    const url = `${GATEWAY_WS_URL}/ws/v1/signal?room=${encodeURIComponent(roomId)}&tenant=${encodeURIComponent(tenantId)}${tokenParam}`;
    const socket = new WebSocket(url);
    activeSocket = socket;

    socket.onopen = () => setStatus("joined");

    socket.onmessage = (event: MessageEvent<string>) => {
      try {
        const frame = JSON.parse(event.data) as SignalFrame;
        onSignalFrame(frame);
      } catch {
        // Non-JSON frame — ignore.
      }
    };

    socket.onerror = () => setError("WebSocket error — check gateway connection.");

    socket.onclose = () => {
      activeSocket = null;
      const { status } = useSignalingStore.getState();
      if (status !== "left") setStatus("idle");
    };
  } catch (err) {
    setError(err instanceof Error ? err.message : "Unknown error opening signal stream.");
  }
}

export function closeSignalStream(): void {
  if (activeSocket) {
    activeSocket.close();
    activeSocket = null;
  }
}
