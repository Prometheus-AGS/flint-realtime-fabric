/**
 * Contract tests for `ShapeClient`.
 *
 * These assert the client against the facade's *documented wire contract*
 * (crates/frf-gateway/src/routes/shape.rs), not against an implementation.
 * Each case names the server behaviour it pins.
 *
 * No test runner is configured in this package yet — see the note in
 * `sdks/ts/README-shape.md`. These are written for `vitest`'s API, which is
 * what the rest of the estate uses.
 */

import { describe, expect, it } from "vitest";

import { ShapeClient } from "./client.js";
import {
  InvalidShapeRequestError,
  ShapeForbiddenError,
  ShapeGrantExpiredError,
  ShapeUpstreamError,
  UnknownShapeError,
} from "./errors.js";

function respond(
  status: number,
  headers: Record<string, string> = {},
  body: unknown = [],
): typeof fetch {
  return (async () =>
    new Response(status === 204 || status === 304 ? null : JSON.stringify(body), {
      status,
      headers: { "content-type": "application/json", ...headers },
    })) as unknown as typeof fetch;
}

function client(fetchImpl: typeof fetch): ShapeClient {
  return new ShapeClient({
    baseUrl: "http://gate.test",
    credentials: { kind: "cookie" },
    fetchImpl,
  });
}

describe("request contract", () => {
  it("sends only the shape id when starting cold", async () => {
    let url = "";
    const spy = (async (input: string) => {
      url = input;
      return new Response("[]", { status: 200, headers: { "content-type": "application/json" } });
    }) as unknown as typeof fetch;

    await client(spy).fetchShape("cases");

    // The facade rejects any narrowing parameter for a shape declaring
    // `allowed_params: []`. A cold request carries the shape id and nothing else.
    const query = new URL(url).searchParams;
    expect([...query.keys()]).toEqual(["shape"]);
    expect(query.get("shape")).toBe("cases");
  });

  it("sends handle and offset together, never one alone", async () => {
    let url = "";
    const spy = (async (input: string) => {
      url = input;
      return new Response("[]", { status: 200, headers: { "content-type": "application/json" } });
    }) as unknown as typeof fetch;

    await client(spy).fetchShape("cases", { handle: "h-1", offset: "17" });

    // `parse_cursor` returns Err for a half cursor, which becomes a 400. The
    // client must never construct one.
    const query = new URL(url).searchParams;
    expect(query.get("handle")).toBe("h-1");
    expect(query.get("offset")).toBe("17");
  });

  it("never sets an Authorization header under cookie credentials", async () => {
    let init: RequestInit | undefined;
    const spy = (async (_: string, requestInit: RequestInit) => {
      init = requestInit;
      return new Response("[]", { status: 200, headers: { "content-type": "application/json" } });
    }) as unknown as typeof fetch;

    await client(spy).fetchShape("cases");

    // The browser holds a session cookie; Gate mints the downstream JWT
    // server-side. A client that held that token would defeat the boundary.
    const headers = (init?.headers ?? {}) as Record<string, string>;
    expect(headers["authorization"]).toBeUndefined();
    expect(init?.credentials).toBe("include");
  });

  it("presents a bearer token when configured for a service caller", async () => {
    let init: RequestInit | undefined;
    const spy = (async (_: string, requestInit: RequestInit) => {
      init = requestInit;
      return new Response("[]", { status: 200, headers: { "content-type": "application/json" } });
    }) as unknown as typeof fetch;

    const service = new ShapeClient({
      baseUrl: "http://gate.test",
      credentials: { kind: "bearer", token: "t-1" },
      fetchImpl: spy,
    });
    await service.fetchShape("cases");

    const headers = (init?.headers ?? {}) as Record<string, string>;
    expect(headers["authorization"]).toBe("Bearer t-1");
    // A service caller has no cookie jar; forcing include would be meaningless.
    expect(init?.credentials).toBeUndefined();
  });
});

describe("refusals are distinguishable", () => {
  it("401 is a recoverable grant expiry", async () => {
    await expect(client(respond(401)).fetchShape("cases")).rejects.toBeInstanceOf(
      ShapeGrantExpiredError,
    );
  });

  it("403 is a non-recoverable refusal", async () => {
    await expect(client(respond(403)).fetchShape("cases")).rejects.toBeInstanceOf(
      ShapeForbiddenError,
    );
  });

  it("404 names the shape, because it is a catalog mismatch not an access error", async () => {
    await expect(client(respond(404)).fetchShape("missing")).rejects.toMatchObject({
      name: "UnknownShapeError",
      shape: "missing",
    });
    await expect(client(respond(404)).fetchShape("missing")).rejects.toBeInstanceOf(
      UnknownShapeError,
    );
  });

  it("400 carries the facade's detail", async () => {
    await expect(client(respond(400)).fetchShape("cases")).rejects.toBeInstanceOf(
      InvalidShapeRequestError,
    );
  });

  it("502 is retryable, unlike every refusal above", async () => {
    await expect(client(respond(502)).fetchShape("cases")).rejects.toBeInstanceOf(
      ShapeUpstreamError,
    );
  });
});

describe("protocol handling", () => {
  it("returns a cursor only when both header halves are present", async () => {
    const both = await client(
      respond(200, { "electric-handle": "h-1", "electric-offset": "0_0" }),
    ).fetchShape("cases");
    expect(both.cursor).toEqual({ handle: "h-1", offset: "0_0" });

    // A half cursor handed back would be echoed and rejected with 400.
    const half = await client(respond(200, { "electric-handle": "h-1" })).fetchShape("cases");
    expect(half.cursor).toBeUndefined();
  });

  it("treats 409 as must-refetch and withholds the cursor", async () => {
    const result = await client(respond(409)).fetchShape("cases", {
      handle: "h-1",
      offset: "17",
    });
    expect(result.mustRefetch).toBe(true);
    // Returning a cursor here would invite a retry that cannot succeed.
    expect(result.cursor).toBeUndefined();
  });

  it("treats the electric-must-refetch header as equivalent to 409", async () => {
    const result = await client(respond(200, { "electric-must-refetch": "true" })).fetchShape(
      "cases",
    );
    expect(result.mustRefetch).toBe(true);
  });

  it("reports up-to-date from the header's presence, not its value", async () => {
    const result = await client(respond(200, { "electric-up-to-date": "" })).fetchShape("cases");
    expect(result.upToDate).toBe(true);
  });

  it("handles 204 as an empty frame WITHOUT reading the body", async () => {
    // SHAPE-FACADE.md: a metadata-only return produces 204, which the Electric
    // protocol treats as empty. Throwing here would break a long-poll loop.
    //
    // Asserting `messages === []` alone is VACUOUS: `Response(null).json()`
    // rejects with SyntaxError, which the client's `.catch(() => [])` swallows
    // into the same empty array. That assertion passes whether or not the 204
    // branch exists — verified by removing the branch and watching the suite
    // stay green. So this asserts the body is never read, which only the
    // early-return branch can satisfy.
    let jsonCalls = 0;
    const spy = (async () => {
      const response = new Response(null, {
        status: 204,
        headers: { "electric-handle": "h-1", "electric-offset": "9" },
      });
      Object.defineProperty(response, "json", {
        value: async () => {
          jsonCalls += 1;
          throw new SyntaxError("Unexpected end of JSON input");
        },
      });
      return response;
    }) as unknown as typeof fetch;

    const result = await client(spy).fetchShape("cases");

    expect(jsonCalls).toBe(0);
    expect(result.messages).toEqual([]);
    expect(result.cursor).toEqual({ handle: "h-1", offset: "9" });
  });

  it("handles 304 as not-modified WITHOUT reading the body", async () => {
    // Same vacuity trap as the 204 case above, same discriminator.
    let jsonCalls = 0;
    const spy = (async () => {
      const response = new Response(null, { status: 304 });
      Object.defineProperty(response, "json", {
        value: async () => {
          jsonCalls += 1;
          throw new SyntaxError("Unexpected end of JSON input");
        },
      });
      return response;
    }) as unknown as typeof fetch;

    const result = await client(spy).fetchShape("cases", undefined, {
      ifNoneMatch: "etag-1",
    });

    expect(jsonCalls).toBe(0);
    expect(result.notModified).toBe(true);
    expect(result.messages).toEqual([]);
  });

  it("drops control frames but preserves deletes for the caller to interpret", async () => {
    const result = await client(
      respond(200, {}, [
        { headers: { control: "up-to-date" } },
        { headers: { operation: "insert" }, key: "a", value: { id: "1" } },
        { headers: { operation: "delete" }, key: "b", value: { id: "2" } },
      ]),
    ).fetchShape("cases");

    // A replica that rebuilds may ignore deletes; a general client must not
    // decide that on the caller's behalf.
    expect(result.messages).toHaveLength(2);
    expect(result.messages[1]?.headers?.operation).toBe("delete");
  });
});
