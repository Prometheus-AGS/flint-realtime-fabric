import type { SpineClient } from "@prometheusags/frf-sdk";
import { EventKind } from "@prometheusags/frf-sdk";
import type { EntityEvent, EntityQuery, EntityRecord } from "./types.js";

const ENTITY_KIND_VALUE = EventKind.ENTITY_CHANGE;

/** Translates entity domain types to/from EventEnvelope wire format. */
export class RealtimeAdapter {
  private readonly client: SpineClient;

  constructor(client: SpineClient) {
    this.client = client;
  }

  /**
   * Subscribe to real-time entity change events matching the query.
   * Filters by entityType when specified via the payload's `entityType` field.
   */
  async *watchEntities(query: EntityQuery): AsyncIterable<EntityEvent> {
    const subscribeArgs: Parameters<SpineClient["subscribe"]>[0] = {
      channelId: query.channelId,
      consumerId: query.consumerId,
    };
    if (query.fromOffset !== undefined) {
      subscribeArgs.from = { value: query.fromOffset };
    }
    const stream = this.client.subscribe(subscribeArgs);

    for await (const envelope of stream) {
      if (envelope.kind !== ENTITY_KIND_VALUE) continue;

      let payload: Record<string, unknown>;
      try {
        payload = JSON.parse(new TextDecoder().decode(envelope.payload)) as Record<string, unknown>;
      } catch {
        continue;
      }

      const entityType = String(payload["entityType"] ?? "");
      if (query.entityType !== undefined && entityType !== query.entityType) {
        continue;
      }

      const event: EntityEvent = {
        entityType,
        entityId: String(payload["entityId"] ?? ""),
        tenantId: envelope.channel?.tenantId ?? "",
        channelId: envelope.channel?.id ?? query.channelId,
        data: (payload["data"] as Record<string, unknown>) ?? {},
        offset: envelope.offset?.value ?? 0n,
      };
      const corrId = envelope.correlationId;
      if (corrId) event.correlationId = corrId;
      yield event;
    }
  }

  /** Publish an entity mutation event to the spine. */
  async mutateEntity(record: EntityRecord): Promise<void> {
    const payload = new TextEncoder().encode(
      JSON.stringify({
        entityType: record.entityType,
        entityId: record.entityId,
        data: record.data,
      }),
    );

    await this.client.publish({
      envelope: {
        id: crypto.randomUUID(),
        channel: {
          id: record.channelId,
          tenantId: record.tenantId,
          path: `entity/${record.entityType}`,
        },
        kind: ENTITY_KIND_VALUE,
        payload,
        correlationId: record.correlationId ?? "",
      },
    });
  }
}
