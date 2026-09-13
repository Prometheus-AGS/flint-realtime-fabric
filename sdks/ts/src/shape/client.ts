/**
 * Client for the FRF authorized shape facade — `GET /v1/shape` (ADR-009).
 *
 * ## Why this is not an ElectricSQL client
 *
 * The request contract is inverted relative to talking to Electric directly.
 * The caller does **not** name the table, the columns, or the predicate:
 *
 *   > Gate and the facade derive allowed rows, columns and practice scope
 *   > server-side from verified identity. A client predicate is useful
 *   > validation but cannot enforce access against a modified client.
 *
 * A request therefore carries a **shape id** plus opaque protocol echoes, and
 * nothing else. A catalog entry with `allowed_params: []` rejects any extra
 * query parameter with 400 — including the `table`, `columns` and `where` that
 * `@electric-sql/client`'s `ShapeStream` always sends. That is why this exists
 * rather than a thin wrapper over that library: it cannot express a request
 * this narrow.
 *
 * ## What the facade preserves
 *
 * Electric's HTTP protocol verbatim — status, `electric-handle`,
 * `electric-offset`, `electric-up-to-date`, `electric-must-refetch`, and the
 * message body. So this parses Electric's wire format while never speaking to
 * Electric, and never holding a credential Electric would accept.
 *
 * ## Credentials
 *
 * Deliberately pluggable. A browser sends its session cookie and lets Gate mint
 * the downstream JWT server-side; a service caller presents its own bearer
 * token. Defaulting to one or the other would make the client unusable in the
 * opposite environment, so `credentials` is explicit at construction.
 */

import {
  InvalidShapeRequestError,
  ShapeForbiddenError,
  ShapeGrantExpiredError,
  ShapeUpstreamError,
  UnknownShapeError,
} from "./errors.js";

/** One Electric protocol frame. Control frames carry no row. */
export interface ShapeMessage {
  readonly headers?: {
    readonly operation?: "insert" | "update" | "delete";
    readonly control?: string;
  };
  readonly key?: string;
  readonly value?: Record<string, unknown>;
}

/**
 * The opaque position in a shape's stream.
 *
 * Both fields are echoed back verbatim on the next request. Neither is
 * interpretable by the client: `offset` is Electric's LSN-derived cursor and
 * `handle` identifies the server-side shape instance. Constructing one by hand
 * is always wrong.
 */
export interface ShapeCursor {
  readonly handle: string;
  readonly offset: string;
}

/** How the caller proves who it is. */
export type ShapeCredentials =
  /** Browser: send the session cookie; Gate mints the downstream token. */
  | { readonly kind: "cookie" }
  /** Service: present a bearer token directly. */
  | { readonly kind: "bearer"; readonly token: string }
  /** Caller-managed headers, for anything else. */
  | { readonly kind: "headers"; readonly headers: Readonly<Record<string, string>> };

export interface ShapeClientOptions {
  /**
   * Gate's base URL — the authorization boundary.
   *
   * Never Electric's own URL. ADR-009 restricts direct Electric access to an
   * operator-loopback diagnostic and never a client path; pointing this at
   * Electric bypasses the boundary entirely.
   */
  readonly baseUrl: string;
  readonly credentials: ShapeCredentials;
  /** Injectable for tests and non-browser runtimes. */
  readonly fetchImpl?: typeof fetch;
}

/** What one `fetchShape` call observed. */
export interface ShapeFetchResult {
  /**
   * Row frames, in upstream order, with control frames removed.
   *
   * Deletes are included here — unlike a replica that rebuilds rather than
   * reconciles, a general client cannot assume the caller has no removal path.
   * Inspect `message.headers.operation` to decide.
   */
  readonly messages: readonly ShapeMessage[];
  /** The cursor to send next, absent when the server returned none. */
  readonly cursor?: ShapeCursor;
  /** The stream has caught up: `electric-up-to-date` was present. */
  readonly upToDate: boolean;
  /**
   * The history this cursor pointed into is gone (`409`, or
   * `electric-must-refetch`). The caller must discard local state for this
   * shape and restart it cold. Retrying with the same cursor cannot succeed.
   */
  readonly mustRefetch: boolean;
  /** `304` — the conditional request matched; nothing changed. */
  readonly notModified: boolean;
}

/** Optional per-request protocol controls the facade accepts. */
export interface ShapeRequestOptions {
  /** Electric long-polling mode (`live=true`). */
  readonly live?: boolean;
  /** Electric cache-busting cursor, echoed from a prior response. */
  readonly cursor?: string;
  /** Conditional request validator, sent as `If-None-Match`. */
  readonly ifNoneMatch?: string;
  /** Cancellation, for a long-poll a caller wants to abandon. */
  readonly signal?: AbortSignal;
}

export class ShapeClient {
  readonly #base: string;
  readonly #credentials: ShapeCredentials;
  readonly #fetch: typeof fetch;

  constructor(options: ShapeClientOptions) {
    this.#base = options.baseUrl.replace(/\/+$/, "");
    this.#credentials = options.credentials;
    this.#fetch = options.fetchImpl ?? globalThis.fetch.bind(globalThis);
  }

  /**
   * Fetch one frame of one shape.
   *
   * Pass `cursor` to continue a stream, or omit it to start cold. The facade
   * refuses a half-formed cursor with 400 rather than guessing, so this only
   * ever sends both fields or neither.
   */
  async fetchShape(
    shape: string,
    cursor?: ShapeCursor,
    options: ShapeRequestOptions = {},
  ): Promise<ShapeFetchResult> {
    const params = new URLSearchParams({ shape });
    // Only protocol echoes accompany the shape id. Anything else is narrowing
    // input, which a shape declaring `allowed_params: []` rejects with 400.
    if (cursor) {
      params.set("handle", cursor.handle);
      params.set("offset", cursor.offset);
    }
    if (options.live === true) params.set("live", "true");
    if (options.cursor !== undefined) params.set("cursor", options.cursor);

    const headers: Record<string, string> = { accept: "application/json" };
    if (options.ifNoneMatch !== undefined) headers["if-none-match"] = options.ifNoneMatch;

    const init: RequestInit = { method: "GET", headers: this.#applyCredentials(headers) };
    if (this.#credentials.kind === "cookie") init.credentials = "include";
    if (options.signal) init.signal = options.signal;

    const response = await this.#fetch(`${this.#base}/v1/shape?${params.toString()}`, init);

    // Refusals first: each is a distinct server decision, and collapsing them
    // into one error is what makes a failed sync undiagnosable.
    if (response.status === 401) throw new ShapeGrantExpiredError();
    if (response.status === 403) throw new ShapeForbiddenError();
    if (response.status === 404) throw new UnknownShapeError(shape);
    if (response.status === 400) {
      throw new InvalidShapeRequestError(await response.text().catch(() => "no detail"));
    }

    const mustRefetch =
      response.status === 409 || response.headers.get("electric-must-refetch") === "true";

    if (!response.ok && !mustRefetch && response.status !== 304) {
      throw new ShapeUpstreamError();
    }

    const nextCursor = this.#readCursor(response.headers);
    const upToDate = response.headers.get("electric-up-to-date") !== null;

    if (mustRefetch) {
      // The caller must restart cold; returning a cursor would invite a retry
      // that cannot succeed.
      return { messages: [], upToDate: false, mustRefetch: true, notModified: false };
    }

    // 304 carries no body; 204 is the facade's "metadata only" reply, which the
    // Electric protocol treats as an empty frame rather than an error.
    if (response.status === 304 || response.status === 204) {
      return {
        messages: [],
        ...(nextCursor ? { cursor: nextCursor } : {}),
        upToDate,
        mustRefetch: false,
        notModified: response.status === 304,
      };
    }

    const body: unknown = await response.json().catch(() => []);
    const messages = Array.isArray(body) ? (body as ShapeMessage[]).filter(isRowOrChange) : [];

    return {
      messages,
      ...(nextCursor ? { cursor: nextCursor } : {}),
      upToDate,
      mustRefetch: false,
      notModified: false,
    };
  }

  #applyCredentials(headers: Record<string, string>): Record<string, string> {
    switch (this.#credentials.kind) {
      case "cookie":
        // The cookie rides on `credentials: "include"`; no header is set, and
        // the client never holds the downstream token.
        return headers;
      case "bearer":
        return { ...headers, authorization: `Bearer ${this.#credentials.token}` };
      case "headers":
        return { ...headers, ...this.#credentials.headers };
    }
  }

  #readCursor(headers: Headers): ShapeCursor | undefined {
    const handle = headers.get("electric-handle");
    const offset = headers.get("electric-offset");
    // Both or neither: a half cursor is exactly what the facade rejects, so
    // never hand one back to a caller who would echo it.
    return handle !== null && offset !== null ? { handle, offset } : undefined;
  }
}

/** Drop control frames; keep anything carrying a row. */
function isRowOrChange(message: ShapeMessage): boolean {
  return message.headers?.control === undefined && message.value !== undefined;
}
