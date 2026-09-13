export { SpineClient } from "./client.js";
export type { ReconnectPolicy } from "./client.js";
export { DEFAULT_RECONNECT_POLICY } from "./client.js";
export { SpineService } from "./gen/flint/v1/envelope_connect.js";
export {
  createSyncClient,
  createAgentClient,
  createSignalClient,
  SyncService,
  AgentService,
  SignalService,
} from "./services.js";
export { EventKind } from "./gen/flint/v1/envelope_pb.js";
export type {
  EventEnvelope,
  PublishRequest,
  PublishResponse,
  SubscribeRequest,
  AckRequest,
  AckResponse,
  Channel,
  Cursor,
  Offset,
} from "./gen/flint/v1/envelope_pb.js";

// Authorized shape facade (ADR-009). Behind the gateway's `shape-facade`
// feature, which is off by default — see docs/SHAPE-FACADE.md.
export { ShapeClient } from "./shape/client.js";
export type {
  ShapeClientOptions,
  ShapeCredentials,
  ShapeCursor,
  ShapeFetchResult,
  ShapeMessage,
  ShapeRequestOptions,
} from "./shape/client.js";
export {
  ShapeError,
  ShapeGrantExpiredError,
  ShapeForbiddenError,
  UnknownShapeError,
  InvalidShapeRequestError,
  ShapeUpstreamError,
} from "./shape/errors.js";
