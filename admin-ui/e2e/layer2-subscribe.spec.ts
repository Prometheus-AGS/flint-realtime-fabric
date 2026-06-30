/**
 * Layer 2 E2E: WebSocket subscribe stream (gateway required).
 *
 * Verifies that the EntitiesPage opens a WebSocket connection to the gateway
 * and the connection-status badge transitions to a non-error state.
 * The AgentActivityPanel on the /#agents route opens its own WS stream.
 *
 * Gate: SKIP_INTEGRATION=true (or unset GATEWAY_URL) skips WS connection tests.
 */

import { test, expect } from "@playwright/test";

const skipIntegration = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];

test.describe("Layer 1: EntitiesPage static shape", () => {
  test("entities page renders without errors", async ({ page }) => {
    await page.goto("/");
    // Page should load — at minimum no hard JS crash
    await expect(page.locator("body")).toBeVisible();
  });

  test("agent panel renders on /#agents", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByRole("heading", { name: "Agent Activity" })).toBeVisible();
  });

  test("agent panel shows empty state message initially", async ({ page }) => {
    await page.goto("/#agents");
    await expect(page.getByText("No agent events yet")).toBeVisible();
  });

  test("agent panel shows connection status badge", async ({ page }) => {
    await page.goto("/#agents");
    // The status span has role="status"
    await expect(page.locator("[role='status']")).toBeVisible();
  });
});

test.describe("Layer 2: WebSocket subscribe (requires gateway)", () => {
  test.skip(skipIntegration, "Set SKIP_INTEGRATION=false and GATEWAY_URL to run Layer 2 tests");

  test("agent panel WS connection status is not 'Error' within 10s", async ({ page }) => {
    await page.goto("/#agents");
    // The status badge should transition to Connecting or Disconnected (not Error)
    // within 10s of the page loading. Error means the WS connect itself failed hard.
    await expect(
      page.locator("[role='status']").filter({ hasText: /Connected|Connecting|Disconnected/i }),
    ).toBeVisible({ timeout: 10_000 });
    // Confirm status is not in error state
    const errorBanner = page.locator("[role='alert']");
    await expect(errorBanner).not.toBeVisible();
  });

  test("entities page loads with no console errors from WS connect", async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleErrors.push(msg.text());
    });
    await page.goto("/");
    await page.waitForTimeout(2_000);
    // Filter out cross-origin or extension errors; only fail on local errors
    const localErrors = consoleErrors.filter(
      (e) => !e.includes("chrome-extension") && !e.includes("moz-extension"),
    );
    expect(localErrors).toHaveLength(0);
  });
});
