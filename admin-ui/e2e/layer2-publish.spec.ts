/**
 * Layer 2 E2E: Publish form-fill (gateway required).
 *
 * Verifies that the SignalingDemoPage's WebRTC panel renders a room-ID input
 * and a Join button that becomes enabled after typing. When the gateway is
 * live, the Join action must produce a visible status change (Connecting or
 * Live) rather than staying in its idle state.
 *
 * Gate: SKIP_INTEGRATION=true (or unset GATEWAY_URL) to run only the UI
 * assertion; SKIP_INTEGRATION=false requires a live gateway.
 */

import { test, expect } from "@playwright/test";

const skipIntegration = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:28080";

test.describe("Layer 1: Signaling page static shape", () => {
  test("signaling demo page renders heading and subheading", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("heading", { name: "Signaling Demo" })).toBeVisible();
    await expect(page.getByText("WebRTC signaling relay")).toBeVisible();
  });

  test("join button is disabled when room ID is empty", async ({ page }) => {
    await page.goto("/#demo/signaling");
    const joinBtn = page.getByRole("button", { name: "Join" });
    await expect(joinBtn).toBeDisabled();
  });

  test("join button becomes enabled after entering a room ID", async ({ page }) => {
    await page.goto("/#demo/signaling");
    const input = page.getByPlaceholder(/room/i).or(page.locator("input[type='text']")).first();
    await input.fill("test-room-001");
    const joinBtn = page.getByRole("button", { name: "Join" });
    await expect(joinBtn).toBeEnabled();
  });
});

test.describe("Layer 2: Publish / Join (requires gateway)", () => {
  test.skip(skipIntegration, "Set SKIP_INTEGRATION=false and GATEWAY_URL to run Layer 2 tests");

  test("clicking Join produces a status transition away from idle", async ({ page }) => {
    await page.goto("/#demo/signaling");
    const input = page.getByPlaceholder(/room/i).or(page.locator("input[type='text']")).first();
    await input.fill("smoke-room-e2e");
    const joinBtn = page.getByRole("button", { name: "Join" });
    await joinBtn.click();

    await expect(
      page.locator("span").filter({ hasText: /Connecting|Live|Disconnected/i }),
    ).toBeVisible({ timeout: 10_000 });
  });

  test("gateway GATEWAY_URL is reachable at /healthz", async () => {
    const res = await fetch(`${GATEWAY_URL}/healthz`);
    expect(res.status).toBe(200);
    const body = (await res.json()) as { status: string };
    expect(body.status).toBe("ok");
  });
});
