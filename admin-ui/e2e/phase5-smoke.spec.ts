/**
 * Phase 5 exit-criterion smoke tests.
 *
 * Exit criterion (RFC-FRF-002 §Phase 5):
 *   AG-UI / A2A / A2UI agent protocols and BossFang (LibreFang) actor bus are
 *   wired into the gateway; the admin-ui streams live agent events via
 *   `GET /ws/v1/agents` and renders them in the AgentActivityPanel.
 *
 * Three layers:
 *   1. UI layer  — verifies static panel shapes; runs in CI without a live gateway.
 *   2. WS layer  — verifies WebSocket connect/auth/disconnect lifecycle; requires
 *                  the gateway to be reachable (SKIP_INTEGRATION env gate).
 *   3. Bus layer — full end-to-end: publish an agent event via the gateway REST
 *                  shim → LibreFang bus → browser WS → AgentActivityPanel row
 *                  (SKIP_INTEGRATION gate; only runs in full-stack environments).
 */

import { test, expect } from "@playwright/test";

const SKIP_INTEGRATION = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:8080";

// ---------------------------------------------------------------------------
// Layer 1 – UI shape (no live gateway required)
// ---------------------------------------------------------------------------

test.describe("Phase 5 UI layer", () => {
  test("agents nav link is present in the header", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("link", { name: "Agents" })).toBeVisible();
  });

  test("agents page renders AgentActivityPanel heading", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByRole("heading", { name: "Agent Activity" })).toBeVisible();
  });

  test("agents page shows connection status element", async ({ page }) => {
    await page.goto("/#agents");
    const status = page.getByRole("status");
    await expect(status).toBeVisible();
  });

  test("agents page shows empty-state message with no events", async ({ page }) => {
    await page.goto("/#agents");
    await expect(
      page.getByText(/No agent events yet/i),
    ).toBeVisible({ timeout: 5000 });
  });

  test("clear button is disabled when there are no events", async ({ page }) => {
    await page.goto("/#agents");
    const btn = page.getByRole("button", { name: /clear agent events/i });
    await expect(btn).toBeDisabled();
  });

  test("navigating to agents page via nav link renders panel", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "Agents" }).click();
    await expect(page).toHaveURL(/#agents/);
    await expect(page.getByRole("heading", { name: "Agent Activity" })).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Layer 2 – WebSocket lifecycle (requires live gateway)
// ---------------------------------------------------------------------------

test.describe("Phase 5 WS layer", () => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to run integration tests");

  test("agent stream WebSocket connects to gateway with demo token", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });
    await expect(page.getByRole("status")).toHaveText(/connected/i);
  });

  test("navigating away closes agent WebSocket gracefully", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });
    await page.goto("/");
    // Entity page should render without crashing.
    await expect(page.getByRole("main")).toBeVisible();
  });

  test("returning to agents tab reconnects WebSocket", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });
    await page.goto("/");
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 20000 });
  });
});

// ---------------------------------------------------------------------------
// Layer 3 – Agent bus end-to-end (requires live gateway + LibreFang bus)
// ---------------------------------------------------------------------------

test.describe("Phase 5 bus end-to-end", () => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to run full-stack agent bus tests");

  test("published agent event appears in AgentActivityPanel", async ({ page, request }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });

    const runId = `smoke-run-${Date.now()}`;

    const resp = await request.post(`${GATEWAY_URL}/v1/agent-event`, {
      headers: { "Content-Type": "application/json" },
      data: {
        agent_id: "smoke-agent-01",
        tenant_id: "test-tenant",
        session_id: "smoke-session",
        protocol: "ag_ui",
        kind: "text_delta",
        run_id: runId,
        content: { type: "text_delta", delta: "hello from smoke test" },
        timestamp: new Date().toISOString(),
      },
    });
    expect(resp.ok()).toBeTruthy();

    // The event row should appear in the activity panel.
    await expect(page.getByText("hello from smoke test")).toBeVisible({ timeout: 10000 });
  });

  test("clear button removes all agent events from the panel", async ({ page, request }) => {
    await page.goto("/#agents");
    await expect(page.getByText("Connected")).toBeVisible({ timeout: 15000 });

    // Publish one event so the panel has content.
    await request.post(`${GATEWAY_URL}/v1/agent-event`, {
      headers: { "Content-Type": "application/json" },
      data: {
        agent_id: "clear-test-agent",
        tenant_id: "test-tenant",
        session_id: "clear-session",
        protocol: "ag_ui",
        kind: "run_start",
        run_id: `clear-run-${Date.now()}`,
        content: { type: "run_start" },
        timestamp: new Date().toISOString(),
      },
    });

    // Wait for it to appear.
    await expect(page.getByRole("list", { name: "Agent events" })).toBeVisible({
      timeout: 10000,
    });

    // Click clear and assert empty state returns.
    await page.getByRole("button", { name: /clear agent events/i }).click();
    await expect(page.getByText(/No agent events yet/i)).toBeVisible();
  });

  test("agent event ring buffer capped at 200 entries (UI smoke)", async ({ page }) => {
    await page.goto("/#agents");
    // Inject 201 synthetic events via page.evaluate to test ring-buffer cap.
    await page.evaluate(() => {
      const dev = (window as unknown as Record<string, unknown>)["__frf_dev"] as
        | Record<string, unknown>
        | undefined;
      const store = dev?.["agentEventStore"];
      if (!store) return;
      const addEvent = (store as { getState: () => { addEvent: (e: unknown) => void } })
        .getState().addEvent;
      for (let i = 0; i < 201; i++) {
        addEvent({
          agent_id: "bench-agent",
          tenant_id: "test-tenant",
          session_id: "bench-session",
          protocol: "ag_ui",
          kind: "text_delta",
          run_id: `bench-run-${i}`,
          content: { type: "text_delta", delta: `event ${i}` },
          timestamp: new Date().toISOString(),
        });
      }
    });

    // The list should show at most 200 items.
    const items = page.getByRole("listitem");
    await expect(items).toHaveCount(200);
  });
});
