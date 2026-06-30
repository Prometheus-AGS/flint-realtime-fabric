import { useEffect, useRef } from "react";
import { useSignalingStore } from "../stores/signalingStore.js";
import { openSignalStream, closeSignalStream } from "../services/signalingService.js";

const DEFAULT_TENANT = import.meta.env["VITE_TENANT_ID"] ?? "00000000-0000-0000-0000-000000000001";

/**
 * Opens a signal stream when `roomId` is set in the store, and closes it
 * when `roomId` is cleared or the component unmounts.
 */
export function useSignalingStream(): void {
  const roomId = useSignalingStore((s) => s.roomId);
  const status = useSignalingStore((s) => s.status);
  const activeRoomRef = useRef<string | null>(null);

  useEffect(() => {
    if (roomId && status === "joining" && activeRoomRef.current !== roomId) {
      activeRoomRef.current = roomId;
      openSignalStream(roomId, DEFAULT_TENANT).catch(() => {
        // Error already dispatched to store via setError.
      });
    }

    if (!roomId) {
      activeRoomRef.current = null;
      closeSignalStream();
    }

    return () => {
      // Cleanup on unmount only — don't close on every status change.
    };
  }, [roomId, status]);

  useEffect(() => {
    return () => {
      closeSignalStream();
    };
  }, []);
}
