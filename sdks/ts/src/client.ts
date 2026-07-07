import { createPromiseClient, type Transport } from "@connectrpc/connect";
import { SpineService } from "./gen/flint/v1/envelope_connect.js";
import {
  EventEnvelope,
  Offset,
  type PublishRequest,
  type PublishResponse,
  SubscribeRequest,
  type AckRequest,
  type AckResponse,
} from "./gen/flint/v1/envelope_pb.js";
import type { PartialMessage } from "@bufbuild/protobuf";

/**
 * Backoff/retry policy for a resilient subscription. Mirrors the Rust SDK's
 * `ReconnectPolicy` so reconnection semantics are identical across languages.
 */
export interface ReconnectPolicy {
  /** Initial delay (ms) before the first reconnect attempt. */
  initialBackoffMs: number;
  /** Maximum delay (ms) between attempts. */
  maxBackoffMs: number;
  /** Multiplier applied after each failed attempt. */
  multiplier: number;
  /** Max consecutive failed reconnects before giving up; `null` = forever. */
  maxRetries: number | null;
}

export const DEFAULT_RECONNECT_POLICY: ReconnectPolicy = {
  initialBackoffMs: 250,
  maxBackoffMs: 30_000,
  multiplier: 2,
  maxRetries: null,
};

const sleep = (ms: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, ms));

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

  /**
   * Reconnecting subscription. On a transport error or a cleanly-closed stream
   * (e.g. a gateway restart) it reconnects with exponential backoff and resumes
   * from `last-seen-offset + 1`, so no event is skipped or replayed. Yields
   * events transparently; a consumer written against this survives restarts
   * without special handling. Throws only if reconnection is exhausted.
   *
   * Semantics mirror the Rust SDK's `resilient_subscribe`.
   */
  async *subscribeResilient(
    req: PartialMessage<SubscribeRequest>,
    policy: ReconnectPolicy = DEFAULT_RECONNECT_POLICY,
  ): AsyncIterable<EventEnvelope> {
    let fromValue: bigint = req.from?.value ?? 0n;
    let backoff = policy.initialBackoffMs;
    let retries = 0;

    for (;;) {
      try {
        const attemptReq = new SubscribeRequest({
          ...req,
          from: new Offset({ value: fromValue }),
        });
        for await (const envelope of this.inner.subscribe(attemptReq)) {
          // Resume AFTER the last delivered offset.
          const seen = envelope.offset?.value ?? fromValue;
          fromValue = seen + 1n;
          yield envelope;
        }
        // Stream ended cleanly (server closed / restart) — reconnect to resume.
        backoff = policy.initialBackoffMs;
        retries = 0;
      } catch (err) {
        retries += 1;
        if (policy.maxRetries !== null && retries > policy.maxRetries) {
          throw new Error(
            `reconnection exhausted after ${retries} attempts: ${String(err)}`,
          );
        }
      }
      await sleep(backoff);
      backoff = Math.min(backoff * policy.multiplier, policy.maxBackoffMs);
    }
  }
}
