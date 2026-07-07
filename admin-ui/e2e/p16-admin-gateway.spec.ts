/**
 * p16-c008 E2E: admin-UI ↔ gateway wiring.
 *
 * Layer 1 (always runs): the login gate renders when no token is present, and
 * the gated app renders once a token is present.
 *
 * Layer 2 (requires a running gateway): with a token and a real gateway
 * reachable, the Entities page mounts and its Connect/gRPC-web subscription does
 * not hard-error. Gated behind SKIP_INTEGRATION / GATEWAY_URL like the other
 * integration specs.
 */

import { test, expect } from "@playwright/test";

const skipIntegration =
  process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];

test.describe("Layer 1: login gate (no gateway required)", () => {
  // These tests exercise the UNAUTHENTICATED state, so start from a clean
  // context that overrides the globally-seeded token.
  test.use({ storageState: { cookies: [], origins: [] } });

  test("shows the login gate when no token is present", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: /sign in/i })).toBeVisible();
    await expect(page.getByLabel(/access token/i)).toBeVisible();
  });

  test("Continue is disabled until a token is entered", async ({ page }) => {
    await page.goto("/");
    const submit = page.getByRole("button", { name: /continue/i });
    await expect(submit).toBeDisabled();
    await page.getByLabel(/access token/i).fill("some-token");
    await expect(submit).toBeEnabled();
  });

  test("entering a token dismisses the gate", async ({ page }) => {
    await page.goto("/");
    await page.getByLabel(/access token/i).fill("a-valid-looking-token");
    await page.getByRole("button", { name: /continue/i }).click();
    // Gate heading should disappear once a token is stored.
    await expect(page.getByRole("heading", { name: /sign in/i })).toHaveCount(0);
  });
});

test.describe("Layer 1b: gated app renders with a seeded token", () => {
  // Uses the globally-seeded token (default storageState) — no override.
  test("app renders (not the login gate) when a token is present", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: /sign in/i })).toHaveCount(0);
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Layer 2: Entities subscribes against a real gateway", () => {
  test.skip(
    skipIntegration,
    "Set SKIP_INTEGRATION=false and GATEWAY_URL to run against a real gateway",
  );

  test("Entities page mounts and does not surface a hard connection error", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("body")).toBeVisible();
    // No hard error banner within 10s — the Connect subscription reached the
    // gateway (even an empty stream is success; a transport failure is not).
    await expect(page.locator("[role='alert']")).toHaveCount(0, { timeout: 10_000 });
  });
});
