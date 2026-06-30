/**
 * Phase 7 exit-criterion smoke tests.
 *
 * Exit criterion (RFC-FRF-002 §Phase 7 — WebRTC + WASM Browser):
 *   - `frf-media-str0m` sovereign SFU adapter compiles and unit tests pass.
 *   - `frf-wasm` WASM SDK exports `AgentStream` and `crdt_apply_delta`.
 *   - `RunAgent` gRPC RPC accepts bidi streaming requests.
 *   - Gateway wires str0m vs LiveKit via `SFU_MODE` env var at runtime.
 *   - Admin-UI SignalingDemoPage renders; transport toggle visible on AgentPanel.
 *   - Dev inject route `/dev/inject-signal` returns 202 in debug builds.
 *   - Playwright E2E gate passes in Dagger CI (Node 24 + Chromium).
 *
 * Three layers:
 *   1. UI layer  — verifies static shapes; runs in CI without a live gateway.
 *   2. WS layer  — verifies WebRTC signaling panel lifecycle (gateway required).
 *   3. gRPC/WASM layer — AgentStream transport toggle (gateway + WASM required).
 */

import { test, expect } from "@playwright/test";

const SKIP_INTEGRATION = !!process.env["SKIP_INTEGRATION"] || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:8080";

// ---------------------------------------------------------------------------
// Layer 1 – UI shape (no live gateway required)
// ---------------------------------------------------------------------------

test.describe("Phase 7 UI layer", () => {
  test("signaling demo page loads with WebRTC panel", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("heading", { name: "Signaling Demo" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "WebRTC Signaling" })).toBeVisible();
  });

  test("signaling panel shows status badge", async ({ page }) => {
    await page.goto("/#demo/signaling");
    // Status badge is always rendered — may be idle, joining, or joined.
    await expect(
      page.locator("span").filter({ hasText: /Not connected|Connecting|Live|Disconnected/i }),
    ).toBeVisible();
  });

  test("signaling panel join button disabled when input is empty", async ({ page }) => {
    await page.goto("/#demo/signaling");
    const joinBtn = page.getByRole("button", { name: "Join" });
    await expect(joinBtn).toBeDisabled();
  });

  test("signaling panel join button enabled after typing room id", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await page.getByLabel("Room ID").fill("test-room-p7");
    const joinBtn = page.getByRole("button", { name: "Join" });
    await expect(joinBtn).toBeEnabled();
  });

  test("agent activity panel renders on #agents hash", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByRole("heading", { name: "Agent Activity" })).toBeVisible();
  });

  test("agent activity panel shows transport toggle button", async ({ page }) => {
    await page.goto("/#agents");
    // Transport toggle shows current mode: "WS" or "gRPC (WASM)"
    const toggle = page.getByRole("button", { name: /current transport/i });
    await expect(toggle).toBeVisible();
  });

  test("transport toggle switches between WS and gRPC", async ({ page }) => {
    await page.goto("/#agents");
    const toggle = page.getByRole("button", { name: /current transport/i });
    await expect(toggle).toContainText("WS");
    await toggle.click();
    await expect(toggle).toContainText("gRPC");
    await toggle.click();
    await expect(toggle).toContainText("WS");
  });

  test("CRDT demo button renders on signaling page", async ({ page }) => {
    await page.goto("/#demo/signaling");
    const crdtBtn = page.getByRole("button", { name: /crdt/i });
    await expect(crdtBtn).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Layer 2 – WebRTC signaling lifecycle (requires gateway)
// ---------------------------------------------------------------------------

test.describe("Phase 7 signaling integration", () => {
  test.skip(SKIP_INTEGRATION, "requires GATEWAY_URL");

  test("gateway /healthz responds 200", async ({ request }) => {
    const resp = await request.get(`${GATEWAY_URL}/healthz`);
    expect(resp.status()).toBe(200);
  });

  test("gateway /dev/inject-signal returns 400 for missing tenant", async ({ request }) => {
    const resp = await request.post(`${GATEWAY_URL}/dev/inject-signal`, {
      data: {
        room_id: "p7-smoke-room",
        from_session: "not-a-uuid",
        tenant_id: "also-not-a-uuid",
        kind: "offer",
        payload: { sdp: "v=0" },
      },
    });
    // 400 — invalid UUID for from_session or tenant_id
    expect(resp.status()).toBe(400);
  });

  test("gateway /dev/inject-signal returns 503 for unknown session", async ({ request }) => {
    const resp = await request.post(`${GATEWAY_URL}/dev/inject-signal`, {
      data: {
        room_id: "p7-smoke-room",
        from_session: "00000000-0000-0000-0000-000000000001",
        tenant_id: "00000000-0000-0000-0000-000000000001",
        kind: "offer",
        payload: { sdp: "v=0" },
      },
    });
    // 503 — session not registered in the SFU
    expect(resp.status()).toBe(503);
  });

  test("signaling panel can join a room via WebSocket", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await page.getByLabel("Room ID").fill("p7-smoke-room");
    await page.getByRole("button", { name: "Join" }).click();
    // Status transitions to "Connecting…" or "Live"
    await expect(
      page.locator("span").filter({ hasText: /Connecting|Live/i }),
    ).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Layer 3 – gRPC/WASM transport (requires gateway + WASM build)
// ---------------------------------------------------------------------------

test.describe("Phase 7 WASM gRPC stream", () => {
  test.skip(
    SKIP_INTEGRATION || !process.env["WASM_AVAILABLE"],
    "requires GATEWAY_URL and WASM_AVAILABLE=1",
  );

  test("switching to gRPC transport shows connecting state", async ({ page }) => {
    await page.goto("/#agents");
    const toggle = page.getByRole("button", { name: /current transport/i });
    await toggle.click(); // switch to gRPC
    await expect(toggle).toContainText("gRPC");
    // Connection status changes (connecting, connected, or error — all acceptable
    // as long as the UI responded to the switch)
    await expect(
      page.locator("[role='status']").filter({ hasText: /Connecting|Connected|Error/i }),
    ).toBeVisible({ timeout: 3000 });
  });
});
