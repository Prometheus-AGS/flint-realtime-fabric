export { SpineClient } from "./client.js";
export { SpineService } from "./gen/flint/v1/envelope_connect.js";
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
  EventKind,
} from "./gen/flint/v1/envelope_pb.js";
