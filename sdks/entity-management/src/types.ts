/** A domain entity record to be published on the spine. */
export interface EntityRecord {
  entityType: string;
  entityId: string;
  tenantId: string;
  channelId: string;
  data: Record<string, unknown>;
  correlationId?: string;
}

/** Filter criteria for subscribing to entity change events. */
export interface EntityQuery {
  channelId: string;
  consumerId: string;
  entityType?: string;
  fromOffset?: bigint;
}

/** A decoded entity change event received from the spine. */
export interface EntityEvent {
  entityType: string;
  entityId: string;
  tenantId: string;
  channelId: string;
  data: Record<string, unknown>;
  offset: bigint;
  correlationId?: string;
}
