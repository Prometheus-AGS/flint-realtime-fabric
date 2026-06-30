import { useEffect } from "react";
import { openGrpcAgentStream, closeGrpcAgentStream } from "../services/agentGrpcStream.js";
import { useAuthStore } from "../../auth/stores/authStore.js";
import { useAgentEventStore } from "../stores/agentEventStore.js";

const DEFAULT_AGENT_ID = import.meta.env["VITE_AGENT_ID"] ?? "default";
const DEFAULT_SESSION_ID = crypto.randomUUID();

/**
 * Opens a Connect-ES gRPC stream to `AgentService.RunAgent` via the WASM SDK.
 *
 * Only active when the store's `transport` is set to `"grpc"`. Falls back
 * silently when the WASM binary is not yet built.
 */
export function useAgentGrpcStream(): void {
  const accessToken = useAuthStore((state) => state.accessToken);
  const transport = useAgentEventStore((s) => s.transport);

  useEffect(() => {
    if (transport !== "grpc") return;
    const token = accessToken ?? "";
    openGrpcAgentStream(token, DEFAULT_AGENT_ID, DEFAULT_SESSION_ID).catch(() => {
      // Error already dispatched to store via setError.
    });
    return () => {
      closeGrpcAgentStream();
    };
  }, [accessToken, transport]);
}
