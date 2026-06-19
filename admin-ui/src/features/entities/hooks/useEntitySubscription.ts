import { useEffect, useRef, useState } from "react";
import { RealtimeAdapter } from "@prometheusags/frf-entity-management";
import type { EntityQuery } from "@prometheusags/frf-entity-management";
import { spineClient } from "../../../infrastructure/gateway.js";
import type { EntityEvent, EntitySubscriptionState } from "../types.js";

const adapter = new RealtimeAdapter(spineClient);

export function useEntitySubscription(query: EntityQuery): EntitySubscriptionState {
  const [events, setEvents] = useState<EntityEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    abortRef.current = new AbortController();
    const ac = abortRef.current;

    async function run(): Promise<void> {
      setConnected(true);
      setError(null);
      try {
        for await (const event of adapter.watchEntities(query)) {
          if (ac.signal.aborted) break;
          setEvents((prev) => [event, ...prev].slice(0, 200));
        }
      } catch (err: unknown) {
        if (!ac.signal.aborted) {
          setError(err instanceof Error ? err.message : "Subscription error");
        }
      } finally {
        setConnected(false);
      }
    }

    void run();

    return () => {
      ac.abort();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query.channelId, query.consumerId, query.entityType]);

  return { events, connected, error };
}
