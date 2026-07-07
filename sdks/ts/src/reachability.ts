/**
 * Compile-time reachability check (p16-c013).
 *
 * This module is type-checked by the SDK build. It proves that every service
 * with a live gateway server is reachable through the SDK: the client factories
 * construct and their RPC methods are callable with the right types. A runtime
 * smoke test against a real gateway is gated behind FRF_GATEWAY_GRPC_URL in the
 * Rust SDK's integration suite; here we assert the TS surface exists and typechecks.
 *
 * It is never executed — `assertReachable` is exported but not called at runtime.
 */

import { createConnectTransport } from "@connectrpc/connect-web";
import { SpineClient } from "./client.js";
import {
  createSyncClient,
  createAgentClient,
  createSignalClient,
} from "./services.js";

export function assertReachable(baseUrl: string): void {
  const transport = createConnectTransport({ baseUrl });

  // SpineService — publish/subscribe/ack + resilient subscribe.
  const spine = SpineClient.create(transport);
  void spine.subscribe;
  void spine.subscribeResilient;

  // SyncService — bidi CRDT sync + checkpoint.
  const sync = createSyncClient(transport);
  void sync.sync;
  void sync.getCheckpoint;

  // AgentService — server-streaming agent runs.
  const agent = createAgentClient(transport);
  void agent.runAgent;

  // SignalService — bidi WebRTC signaling.
  const signal = createSignalClient(transport);
  void signal.signal;
}
