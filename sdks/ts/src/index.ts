export { SpineClient } from "./client.js";
export { SpineService } from "./gen/flint/v1/envelope_connect.js";
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
