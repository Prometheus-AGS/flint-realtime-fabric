/**
 * Media E2E (G1): a real browser peer connects to the sovereign SFU (p23-c003).
 *
 * A real Chromium `RTCPeerConnection` negotiates offer/answer over the gateway's
 * `/ws/v1/signal` WebSocket and observes ICE reaching `connected` — the first genuine
 * browser-side proof of the sovereign media plane (phases 21–22 proved it in-process).
 *
 * HONEST GATE — this never reports a false green:
 *   - UI-shape assertions always run.
 *   - The real-connection test is `test.skip`-gated on SKIP_INTEGRATION / GATEWAY_URL.
 *     With no live SFU_MODE=sovereign gateway it is SKIPPED, not passed.
 *
 * Requires a running gateway with SFU_MODE=sovereign (WS inbound path, p23-c003 task 1).
 * CI wiring: run behind a browser-available flag in Dagger (Node + Chromium); where CI
 * lacks Chromium, run locally and record it as locally-proven — do not silently skip the
 * whole file.
 */

import { test, expect } from "@playwright/test";
import { connectToSovereignSfu } from "./support/webrtc-client.js";

const skipIntegration =
  process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:28080";
const WS_URL = GATEWAY_URL.replace(/^http/, "ws");
const TENANT = process.env["E2E_TENANT_ID"] ?? "00000000-0000-0000-0000-000000000000";
const TOKEN = process.env["E2E_JWT"];

test.describe("Media: signaling page shape (always runs)", () => {
  test("signaling demo page renders", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("heading", { name: "Signaling Demo" })).toBeVisible();
  });
});

test.describe("Media: browser peer reaches Connected on the sovereign SFU (requires gateway)", () => {
  test.skip(
    skipIntegration,
    "Set SKIP_INTEGRATION=false + GATEWAY_URL to a running SFU_MODE=sovereign gateway.",
  );

  test("gateway is reachable at /healthz", async () => {
    const res = await fetch(`${GATEWAY_URL}/healthz`);
    expect(res.ok).toBeTruthy();
  });

  test("RTCPeerConnection negotiates via /ws/v1/signal and reaches connected", async ({
    page,
  }) => {
    await page.goto("/#demo/signaling");

    const result = await page.evaluate(connectToSovereignSfu, {
      wsUrl: WS_URL,
      room: "e2e-media-room",
      tenant: TENANT,
      token: TOKEN,
      timeoutMs: 15_000,
    });

    expect(
      result.connected,
      `ICE terminal state was "${result.state}" — expected "connected"`,
    ).toBeTruthy();
  });
});
