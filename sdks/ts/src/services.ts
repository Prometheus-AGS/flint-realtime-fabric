import { createPromiseClient, type Transport } from "@connectrpc/connect";
import { SyncService } from "./gen/flint/v1/sync_connect.js";
import { AgentService } from "./gen/flint/v1/agent_connect.js";
import { SignalService } from "./gen/flint/v1/signal_connect.js";
import { EntityService } from "./gen/flint/v1/entity_connect.js";
import { AuthzService } from "./gen/flint/v1/authz_connect.js";

/**
 * Thin typed clients for the gateway services beyond SpineService.
 *
 * All of these now have a live gateway server implementation: Sync (CRDT sync),
 * Agent (agent runs), Signal (WebRTC signaling), Entity (read/watch, live since
 * p17-c004), and Authz (relation check/write/delete, live since p17-c005).
 */

export const createSyncClient = (transport: Transport) =>
  createPromiseClient(SyncService, transport);

export const createAgentClient = (transport: Transport) =>
  createPromiseClient(AgentService, transport);

export const createSignalClient = (transport: Transport) =>
  createPromiseClient(SignalService, transport);

export const createEntityClient = (transport: Transport) =>
  createPromiseClient(EntityService, transport);

export const createAuthzClient = (transport: Transport) =>
  createPromiseClient(AuthzService, transport);

export { SyncService, AgentService, SignalService, EntityService, AuthzService };
