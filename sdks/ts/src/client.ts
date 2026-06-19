import { createPromiseClient, type Transport } from "@connectrpc/connect";
import { SpineService } from "./gen/flint/v1/envelope_connect.js";
import type {
  EventEnvelope,
  PublishRequest,
  PublishResponse,
  SubscribeRequest,
  AckRequest,
  AckResponse,
} from "./gen/flint/v1/envelope_pb.js";
import type { PartialMessage } from "@bufbuild/protobuf";

export class SpineClient {
  private readonly inner: ReturnType<typeof createPromiseClient<typeof SpineService>>;

  private constructor(transport: Transport) {
    this.inner = createPromiseClient(SpineService, transport);
  }

  static create(transport: Transport): SpineClient {
    return new SpineClient(transport);
  }

  publish(envelope: PartialMessage<PublishRequest>): Promise<PublishResponse> {
    return this.inner.publish(envelope);
  }

  subscribe(req: PartialMessage<SubscribeRequest>): AsyncIterable<EventEnvelope> {
    return this.inner.subscribe(req);
  }

  ack(req: PartialMessage<AckRequest>): Promise<AckResponse> {
    return this.inner.ack(req);
  }
}
