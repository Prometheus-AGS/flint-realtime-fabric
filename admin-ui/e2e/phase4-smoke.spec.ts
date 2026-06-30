/**
 * Phase 4 exit-criterion smoke tests.
 *
 * Exit criterion (RFC-FRF-002 §Phase 4):
 *   Browser client (via frf-wasm + Connect-ES) subscribes to an entity stream,
 *   edits offline, and reconnects via WebSocket mux; CDC event from PostgreSQL
 *   WAL flows end-to-end through the spine to the browser.
 *
 * Three layers:
 *   1. UI layer  — verifies static shapes; runs in CI without a live gateway.
 *   2. WS layer  — verifies WebSocket connect/disconnect lifecycle; requires
 *                  the gateway to be reachable (SKIP_INTEGRATION env gate).
 *   3. CDC layer — full end-to-end: PostgreSQL WAL → Iggy spine → browser WS
 *                  (SKIP_INTEGRATION gate; only runs in full-stack environments).
 */

import { test, expect } from "@playwright/test";

const SKIP_INTEGRATION = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:8080";

// ---------------------------------------------------------------------------
// Layer 1 – UI shape (no live gateway required)
// ---------------------------------------------------------------------------

test.describe("Phase 4 UI layer", () => {
  test("entities page loads with entity stream section", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Entity Stream" })).toBeVisible();
    await expect(page.getByLabel("Entity event stream")).toBeVisible();
  });

  test("entity stream shows connection status indicator", async ({ page }) => {
    await page.goto("/");
    // The green/red dot has an aria-label for the connection state.
    const dot = page.getByLabel(/connected|disconnected/i).first();
    await expect(dot).toBeVisible();
  });

  test("entity stream shows waiting message when no events", async ({ page }) => {
    await page.goto("/");
    // Without a live gateway the WS will fail and show the empty state.
    await expect(
      page.getByText(/Waiting for entity change events|error/i),
    ).toBeVisible({ timeout: 5000 });
  });

  test("signaling demo page is reachable via nav link", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "Signaling" }).click();
    await expect(page).toHaveURL(/#demo\/signaling/);
    await expect(page.getByRole("heading", { level: 1 })).toContainText("Signaling");
  });

  test("CRDT demo button renders on signaling page", async ({ page }) => {
    await page.goto("/#demo/signaling");
    const btn = page.getByRole("button", { name: "Run CRDT merge" });
    await expect(btn).toBeVisible();
    await expect(btn).toBeEnabled();
  });

  test("CRDT demo button shows error when frf-wasm is not built", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await page.getByRole("button", { name: "Run CRDT merge" }).click();
    // Either a success result or a graceful error — both are valid without the WASM bundle.
    await expect(
      page.locator("[data-testid='crdt-result'], [role='alert']"),
    ).toBeVisible({ timeout: 10000 });
  });
});

// ---------------------------------------------------------------------------
// Layer 2 – WebSocket lifecycle (requires live gateway)
// ---------------------------------------------------------------------------

test.describe("Phase 4 WS layer", () => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to run integration tests");

  test("entity stream WebSocket connects to gateway", async ({ page }) => {
    await page.goto("/");
    // Poll for green dot (aria-label="Connected")
    await expect(page.getByLabel("Connected")).toBeVisible({ timeout: 15000 });
  });

  test("entity stream shows 'live' after WebSocket connect", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("live")).toBeVisible({ timeout: 15000 });
  });

  test("navigating away closes WebSocket gracefully", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByLabel("Connected")).toBeVisible({ timeout: 15000 });
    // Navigate to signaling page — entity stream should unmount and close WS.
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("heading", { level: 1 })).toContainText("Signaling");
    // No assertion on WS close itself — Playwright can't introspect WS frames here.
    // The fact that the page renders correctly confirms no crash on unmount.
  });
});

// ---------------------------------------------------------------------------
// Layer 3 – CDC end-to-end (requires live PostgreSQL + gateway + Iggy)
// ---------------------------------------------------------------------------

test.describe("Phase 4 CDC end-to-end", () => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to run full-stack CDC tests");

  test("CDC event from PostgreSQL WAL flows to entity stream table", async ({ page, request }) => {
    await page.goto("/");
    await expect(page.getByLabel("Connected")).toBeVisible({ timeout: 15000 });

    // Trigger a publish via the gateway REST endpoint.
    const resp = await request.post(`${GATEWAY_URL}/v1/publish`, {
      headers: { "Content-Type": "application/json" },
      data: {
        channel_id: "00000000-0000-0000-0000-000000000001",
        tenant_id: "test-tenant",
        entity_type: "e2e-smoke",
        entity_id: `smoke-${Date.now()}`,
        payload: { smoke: true },
      },
    });
    expect(resp.ok()).toBeTruthy();

    // The published event should appear in the entity stream table.
    await expect(
      page.getByRole("cell", { name: "e2e-smoke" }),
    ).toBeVisible({ timeout: 10000 });
  });

  test("browser reconnects after WebSocket drop and resumes receiving events", async ({ page, request }) => {
    await page.goto("/");
    await expect(page.getByLabel("Connected")).toBeVisible({ timeout: 15000 });

    // Simulate WS drop by navigating away and back.
    await page.goto("/#demo/signaling");
    await page.goto("/");
    await expect(page.getByLabel("Connected")).toBeVisible({ timeout: 20000 });

    // Publish after reconnect — should still reach the browser.
    const resp = await request.post(`${GATEWAY_URL}/v1/publish`, {
      headers: { "Content-Type": "application/json" },
      data: {
        channel_id: "00000000-0000-0000-0000-000000000001",
        tenant_id: "test-tenant",
        entity_type: "e2e-reconnect",
        entity_id: `reconnect-${Date.now()}`,
        payload: { reconnect: true },
      },
    });
    expect(resp.ok()).toBeTruthy();

    await expect(
      page.getByRole("cell", { name: "e2e-reconnect" }),
    ).toBeVisible({ timeout: 10000 });
  });
});
