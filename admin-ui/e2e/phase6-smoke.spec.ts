/**
 * Phase 6 exit-criterion smoke tests.
 *
 * Exit criterion (RFC-FRF-002 §Phase 6):
 *   Matrix and ATProto federation bridges are wired into the gateway as
 *   background ingest tasks; federated events arrive on the spine and are
 *   visible in the admin-ui agent activity panel.
 *
 * Three layers:
 *   1. UI layer   — verifies static panel shapes and synthetic event injection;
 *                   runs in CI without a live gateway.
 *   2. gRPC layer — verifies the gRPC `RunAgent` endpoint is reachable and
 *                   bridge tasks start without error (SKIP_INTEGRATION gate).
 *   3. Bus layer  — full end-to-end: inject a Matrix event via the dev
 *                   injection endpoint → spine → WS → admin-ui panel
 *                   (SKIP_INTEGRATION gate; only runs in full-stack environments).
 */

import { test, expect } from "@playwright/test";

const SKIP_INTEGRATION = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:8080";

// ---------------------------------------------------------------------------
// Layer 1 – UI shape (no live gateway required)
// ---------------------------------------------------------------------------

test.describe("Phase 6 UI layer", () => {
  test("agents page renders activity panel heading", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByRole("heading", { name: "Agent Activity" })).toBeVisible();
  });

  test("agents page shows empty-state message when store has no events", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByText(/No agent events yet/i)).toBeVisible({ timeout: 5000 });
  });

  test("synthetic Matrix protocol event appears in panel after store injection", async ({
    page,
  }) => {
    await page.goto("/#agents");

    await page.evaluate(() => {
      const dev = (window as unknown as Record<string, unknown>)["__frf_dev"] as
        | Record<string, unknown>
        | undefined;
      const store = dev?.["agentEventStore"];
      if (!store) return;
      const { addEvent } = (
        store as { getState: () => { addEvent: (e: unknown) => void } }
      ).getState();
      addEvent({
        agent_id: "00000000-0000-0000-0000-000000000001",
        tenant_id: "00000000-0000-0000-0000-000000000002",
        session_id: "00000000-0000-0000-0000-000000000003",
        protocol: "ag_ui",
        kind: "text_delta",
        run_id: "matrix-smoke-run",
        content: { type: "text_delta", delta: "federated via Matrix" },
        timestamp: new Date().toISOString(),
      });
    });

    await expect(page.getByText("federated via Matrix")).toBeVisible({ timeout: 3000 });
  });

  test("synthetic ATProto protocol event appears in panel after store injection", async ({
    page,
  }) => {
    await page.goto("/#agents");

    await page.evaluate(() => {
      const dev = (window as unknown as Record<string, unknown>)["__frf_dev"] as
        | Record<string, unknown>
        | undefined;
      const store = dev?.["agentEventStore"];
      if (!store) return;
      const { addEvent } = (
        store as { getState: () => { addEvent: (e: unknown) => void } }
      ).getState();
      addEvent({
        agent_id: "00000000-0000-0000-0000-000000000004",
        tenant_id: "00000000-0000-0000-0000-000000000002",
        session_id: "00000000-0000-0000-0000-000000000005",
        protocol: "ag_ui",
        kind: "text_delta",
        run_id: "atproto-smoke-run",
        content: { type: "text_delta", delta: "federated via ATProto" },
        timestamp: new Date().toISOString(),
      });
    });

    await expect(page.getByText("federated via ATProto")).toBeVisible({ timeout: 3000 });
  });

  test("dev injection endpoint returns 404 in release builds (or is absent)", async ({
    request,
  }) => {
    // In CI without a gateway this is skipped; the route only exists in debug builds.
    test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to verify dev endpoint absence in release");

    const resp = await request.post(`${GATEWAY_URL}/dev/inject-federation-event`, {
      headers: { "Content-Type": "application/json" },
      data: {},
      failOnStatusCode: false,
    });
    // Release build must return 404 — the route is #[cfg(debug_assertions)] only.
    expect(resp.status()).toBe(404);
  });
});

// ---------------------------------------------------------------------------
// Layer 2 – gRPC / bridge lifecycle (requires live gateway)
// ---------------------------------------------------------------------------

test.describe("Phase 6 gRPC layer", () => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to run integration tests");

  test("gRPC port 9090 is reachable (HTTP/2 handshake succeeds)", async ({ request }) => {
    const grpcUrl = GATEWAY_URL.replace(/:(\d+)$/, ":9090");
    // tonic serves HTTP/2; a plain HTTP GET returns 200 or 400 — either proves reachability.
    const resp = await request.get(grpcUrl, { failOnStatusCode: false });
    expect([200, 400, 415]).toContain(resp.status());
  });

  test("gateway /healthz is still up after federation bridges start", async ({ request }) => {
    const resp = await request.get(`${GATEWAY_URL}/healthz`);
    expect(resp.ok()).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Layer 3 – Federation bus end-to-end (requires full stack + dev injection)
// ---------------------------------------------------------------------------

test.describe("Phase 6 federation bus end-to-end", () => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to run full-stack federation smoke tests");

  test("Matrix event injected via dev endpoint appears in agent stream WebSocket", async ({
    page,
    request,
  }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });

    const runId = `matrix-e2e-${String(Date.now())}`;

    const injectResp = await request.post(`${GATEWAY_URL}/dev/inject-federation-event`, {
      headers: { "Content-Type": "application/json" },
      data: {
        protocol: "matrix",
        source: "!smoke-room:matrix.org",
        content: { type: "text_delta", delta: `matrix smoke ${runId}` },
      },
    });
    expect(injectResp.ok()).toBeTruthy();

    await expect(page.getByText(`matrix smoke ${runId}`)).toBeVisible({ timeout: 5000 });
  });

  test("ATProto event injected via dev endpoint appears in agent stream WebSocket", async ({
    page,
    request,
  }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });

    const runId = `atproto-e2e-${String(Date.now())}`;

    const injectResp = await request.post(`${GATEWAY_URL}/dev/inject-federation-event`, {
      headers: { "Content-Type": "application/json" },
      data: {
        protocol: "atproto",
        source: "did:plc:smoke-test",
        content: { type: "text_delta", delta: `atproto smoke ${runId}` },
      },
    });
    expect(injectResp.ok()).toBeTruthy();

    await expect(page.getByText(`atproto smoke ${runId}`)).toBeVisible({ timeout: 5000 });
  });
});
