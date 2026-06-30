import { useAgentEventStore } from "../stores/agentEventStore.js";
import type { AgentEvent } from "../types/agent.js";

const GATEWAY_WS_URL = (import.meta.env["VITE_GATEWAY_URL"] ?? "http://localhost:4000")
  .replace(/^http/, "ws");

const RECONNECT_DELAYS_MS = [1000, 2000, 4000, 8000, 16000];

let activeSocket: WebSocket | null = null;
let reconnectAttempt = 0;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let isStopped = false;

function scheduleReconnect(token: string): void {
  if (isStopped) return;
  const delay = RECONNECT_DELAYS_MS[Math.min(reconnectAttempt, RECONNECT_DELAYS_MS.length - 1)] ?? 16000;
  reconnectAttempt += 1;
  reconnectTimer = setTimeout(() => {
    if (!isStopped) openAgentStream(token);
  }, delay);
}

export function openAgentStream(token: string): void {
  closeAgentStream();
  isStopped = false;

  const { setConnectionStatus, setError, addEvent } = useAgentEventStore.getState();
  setConnectionStatus("connecting");

  try {
    const url = `${GATEWAY_WS_URL}/ws/v1/agents`;
    const socket = new WebSocket(url, undefined);
    activeSocket = socket;

    socket.onopen = () => {
      reconnectAttempt = 0;
      setConnectionStatus("connected");
      socket.send(JSON.stringify({ type: "auth", token }));
    };

    socket.onmessage = (event: MessageEvent<string>) => {
      try {
        const agentEvent = JSON.parse(event.data) as AgentEvent;
        addEvent(agentEvent);
      } catch {
        // Non-JSON frame — ignore silently.
      }
    };

    socket.onerror = () => {
      setError("WebSocket error — check gateway connection.");
    };

    socket.onclose = () => {
      activeSocket = null;
      if (!isStopped) {
        setConnectionStatus("disconnected");
        scheduleReconnect(token);
      }
    };
  } catch (err) {
    setError(err instanceof Error ? err.message : "Unknown error opening agent stream.");
  }
}

export function closeAgentStream(): void {
  isStopped = true;
  if (reconnectTimer !== null) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  if (activeSocket) {
    activeSocket.close();
    activeSocket = null;
  }
  reconnectAttempt = 0;
}
