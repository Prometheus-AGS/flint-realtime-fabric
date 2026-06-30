import { create } from "zustand";
import type { AgentEvent } from "../types/agent.js";

const MAX_EVENTS = 200;

export type AgentTransport = "websocket" | "grpc";

interface AgentEventState {
  events: AgentEvent[];
  connectionStatus: "disconnected" | "connecting" | "connected" | "error";
  lastError: string | null;
  transport: AgentTransport;
}

interface AgentEventActions {
  addEvent: (event: AgentEvent) => void;
  setConnectionStatus: (status: AgentEventState["connectionStatus"]) => void;
  setError: (error: string) => void;
  clearEvents: () => void;
  setTransport: (transport: AgentTransport) => void;
}

export type AgentEventStore = AgentEventState & AgentEventActions;

export const useAgentEventStore = create<AgentEventStore>((set) => ({
  events: [],
  connectionStatus: "disconnected",
  lastError: null,
  transport: "websocket",

  addEvent: (event) =>
    set((state) => ({
      events: [event, ...state.events].slice(0, MAX_EVENTS),
    })),

  setConnectionStatus: (connectionStatus) => set({ connectionStatus }),

  setError: (error) => set({ lastError: error, connectionStatus: "error" }),

  clearEvents: () => set({ events: [] }),

  setTransport: (transport) => set({ transport }),
}));

if (import.meta.env.DEV) {
  const w = window as unknown as Record<string, unknown>;
  w.__frf_dev = {
    ...((w.__frf_dev as object) ?? {}),
    agentEventStore: useAgentEventStore,
  };
}
