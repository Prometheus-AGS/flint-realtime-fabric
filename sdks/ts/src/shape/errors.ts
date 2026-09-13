/**
 * Typed refusals from the authorized shape facade (ADR-009).
 *
 * Every one of these is a *server decision*, not a transport failure. They are
 * separate classes because a consumer must react differently to each: a grant
 * that expired can be recovered by re-establishing the session, a forbidden
 * shape cannot, and an unknown shape is a deployment/catalog mismatch that no
 * amount of retrying will fix.
 */

/** Base class, so `catch (e) { if (e instanceof ShapeError) ... }` works. */
export class ShapeError extends Error {
  constructor(
    message: string,
    /** The HTTP status the facade returned. */
    readonly status: number,
  ) {
    super(message);
    this.name = "ShapeError";
  }
}

/**
 * `401` — the grant backing this request has expired.
 *
 * Recoverable: re-establish the session and retry. This is NOT a transient
 * network error, so it must not be swallowed by a backoff loop that assumes
 * one; the credential itself has to change.
 */
export class ShapeGrantExpiredError extends ShapeError {
  constructor() {
    super("shape grant expired — the session must be re-established", 401);
    this.name = "ShapeGrantExpiredError";
  }
}

/**
 * `403` — authorization refused, or the continuation handle does not belong to
 * this grant.
 *
 * The facade returns 403 for both `Unauthorized` and `HandleMismatch`, and
 * deliberately does not distinguish them on the wire: telling a caller *which*
 * of the two occurred would leak whether a handle exists. Not recoverable by
 * retrying the same request.
 */
export class ShapeForbiddenError extends ShapeError {
  constructor() {
    super("shape access forbidden — the grant does not cover this request", 403);
    this.name = "ShapeForbiddenError";
  }
}

/**
 * `404` — the shape id is not in the server's catalog.
 *
 * A deployment error, not an access error. The client and server disagree
 * about which shapes exist.
 */
export class UnknownShapeError extends ShapeError {
  constructor(readonly shape: string) {
    super(`unknown shape '${shape}' — not declared in the server catalog`, 404);
    this.name = "UnknownShapeError";
  }
}

/**
 * `400` — the request was malformed.
 *
 * In practice this means one of two things: a half-formed cursor (a `handle`
 * without an `offset`, which the facade refuses rather than guessing at), or a
 * narrowing parameter the shape's `allowed_params` does not permit. The
 * facade's message is carried through because it names the offending key class
 * without disclosing values.
 */
export class InvalidShapeRequestError extends ShapeError {
  constructor(detail: string) {
    super(`invalid shape request: ${detail}`, 400);
    this.name = "InvalidShapeRequestError";
  }
}

/**
 * `502` — the facade reached Electric and the exchange failed upstream.
 *
 * Distinct from the refusals above: nothing about the grant is wrong, so this
 * *is* a candidate for retry with backoff.
 */
export class ShapeUpstreamError extends ShapeError {
  constructor() {
    super("shape upstream failed — the facade could not complete the exchange", 502);
    this.name = "ShapeUpstreamError";
  }
}
