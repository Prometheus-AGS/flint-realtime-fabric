import type { EntityEvent } from "@prometheusags/frf-entity-management";

export type { EntityEvent };

export interface EntitySubscriptionState {
  events: EntityEvent[];
  connected: boolean;
  error: string | null;
}
