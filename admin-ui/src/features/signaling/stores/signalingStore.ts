import { create } from "zustand";

export interface SignalFrame {
  fromSession: string;
  toSession?: string;
  tenantId: string;
  roomId: string;
  kind: SignalKind;
  sfuMode: "hosted" | "sovereign";
  timestamp: string;
}

export type SignalKind =
  | "offer"
  | "answer"
  | "ice_candidate"
  | "ice_restart"
  | "hangup"
  | "room_join"
  | "room_leave";

export type SignalingStatus = "idle" | "joining" | "joined" | "left" | "error";

interface SignalingState {
  roomId: string | null;
  status: SignalingStatus;
  sfuMode: "hosted" | "sovereign";
  participants: string[];
  lastError: string | null;
  frames: SignalFrame[];
}

interface SignalingActions {
  joinRoom: (roomId: string) => void;
  leaveRoom: () => void;
  onSignalFrame: (frame: SignalFrame) => void;
  setStatus: (status: SignalingStatus) => void;
  setError: (error: string) => void;
}

export type SignalingStore = SignalingState & SignalingActions;

export const useSignalingStore = create<SignalingStore>((set) => ({
  roomId: null,
  status: "idle",
  sfuMode: "hosted",
  participants: [],
  lastError: null,
  frames: [],

  joinRoom: (roomId) =>
    set({ roomId, status: "joining", participants: [], lastError: null, frames: [] }),

  leaveRoom: () =>
    set({ roomId: null, status: "left", participants: [] }),

  onSignalFrame: (frame) =>
    set((state) => {
      const participants = state.participants.includes(frame.fromSession)
        ? state.participants
        : [...state.participants, frame.fromSession];

      return {
        frames: [frame, ...state.frames.slice(0, 49)],
        participants,
        status: frame.kind === "room_join" ? "joined" : state.status,
      };
    }),

  setStatus: (status) => set({ status }),

  setError: (error) => set({ lastError: error, status: "error" }),
}));
