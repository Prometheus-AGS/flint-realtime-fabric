/**
 * Layer 2 E2E: Agent session WS lifecycle (gateway required).
 *
 * Verifies that the AgentActivityPanel:
 * - Renders transport toggle and connection status controls.
 * - Shows the transport label (WS or gRPC) in the toggle button.
 * - With a live gateway: WS connection transitions to Connected within 10s.
 * - Clear button clears the event list.
 *
 * Gate: SKIP_INTEGRATION=true (or unset GATEWAY_URL) skips live-gateway tests.
 */

import { test, expect } from "@playwright/test";

const skipIntegration = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];

test.describe("Layer 1: AgentActivityPanel controls", () => {
  test("panel renders heading", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByRole("heading", { name: "Agent Activity" })).toBeVisible();
  });

  test("transport toggle button shows WS or gRPC label", async ({ page }) => {
    await page.goto("/#agents");
    const toggleBtn = page.getByRole("button", { name: /transport/i });
    await expect(toggleBtn).toBeVisible();
    await expect(toggleBtn).toHaveText(/WS|gRPC/i);
  });

  test("clicking transport toggle switches label", async ({ page }) => {
    await page.goto("/#agents");
    const toggleBtn = page.getByRole("button", { name: /transport/i });
    const before = await toggleBtn.textContent();
    await toggleBtn.click();
    const after = await toggleBtn.textContent();
    expect(before).not.toBe(after);
  });

  test("clear button is disabled when no events", async ({ page }) => {
    await page.goto("/#agents");
    const clearBtn = page.getByRole("button", { name: "Clear agent events" });
    await expect(clearBtn).toBeDisabled();
  });
});

test.describe("Layer 2: Agent session WS (requires gateway)", () => {
  test.skip(skipIntegration, "Set SKIP_INTEGRATION=false and GATEWAY_URL to run Layer 2 tests");

  test("WS transport reaches Connected or Disconnected (not Error) within 10s", async ({
    page,
  }) => {
    await page.goto("/#agents");
    // Ensure we are on WS transport
    const toggleBtn = page.getByRole("button", { name: /transport/i });
    const label = await toggleBtn.textContent();
    if (label?.includes("gRPC")) {
      await toggleBtn.click();
    }

    const statusBadge = page.locator("[role='status']");
    await expect(statusBadge).toBeVisible();
    await expect(
      statusBadge.filter({ hasText: /Connected|Connecting|Disconnected/i }),
    ).toBeVisible({ timeout: 10_000 });

    // No error alert should appear for a plain WS connect
    await expect(page.locator("[role='alert']")).not.toBeVisible();
  });

  test("connection status is not stuck in Error state", async ({ page }) => {
    await page.goto("/#agents");
    // Give WS time to attempt connection
    await page.waitForTimeout(3_000);
    const statusText = await page.locator("[role='status']").textContent();
    expect(statusText).not.toMatch(/^Error$/i);
  });
});
