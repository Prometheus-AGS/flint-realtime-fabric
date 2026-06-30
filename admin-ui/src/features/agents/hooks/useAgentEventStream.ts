import { useEffect } from "react";
import { openAgentStream, closeAgentStream } from "../services/agentWebSocket.js";
import { useAuthStore } from "../../auth/stores/authStore.js";
import { useAgentEventStore } from "../stores/agentEventStore.js";

export function useAgentEventStream(): void {
  const accessToken = useAuthStore((state) => state.accessToken);
  const transport = useAgentEventStore((s) => s.transport);

  useEffect(() => {
    if (transport !== "websocket") return;
    const token = accessToken ?? "";
    openAgentStream(token);
    return () => {
      closeAgentStream();
    };
  }, [accessToken, transport]);
}
