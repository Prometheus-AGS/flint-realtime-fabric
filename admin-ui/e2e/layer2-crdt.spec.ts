/**
 * Layer 2 E2E: CRDT apply_delta via WebAssembly (gateway not required).
 *
 * Layer 1 tests verify the UI shape of the CrdtDemoButton component on the
 * SignalingDemoPage. No gateway or built WASM is needed.
 *
 * Layer 2 tests exercise the actual `crdt_apply_delta` export by triggering
 * the "Run CRDT merge" button with WASM loaded. Gated on WASM_AVAILABLE=1
 * and SKIP_INTEGRATION=false.
 *
 * Gate: SKIP_INTEGRATION=true (or WASM_AVAILABLE unset) to run only Layer 1;
 *       SKIP_INTEGRATION=false + WASM_AVAILABLE=1 to also run Layer 2.
 */

import { test, expect } from "@playwright/test";

const skipIntegration = process.env["SKIP_INTEGRATION"] === "true"
    || !process.env["WASM_AVAILABLE"];

test.describe("Layer 1: CRDT demo component static shape", () => {
    test("signaling demo page renders the CRDT demo section", async ({ page }) => {
        await page.goto("/#demo/signaling");
        await expect(page.getByRole("heading", { name: "CRDT Demo" })).toBeVisible();
    });

    test("CRDT demo renders the Run CRDT merge button", async ({ page }) => {
        await page.goto("/#demo/signaling");
        await expect(page.getByRole("button", { name: "Run CRDT merge" })).toBeVisible();
    });

    test("Run CRDT merge button is enabled in idle state", async ({ page }) => {
        await page.goto("/#demo/signaling");
        await expect(page.getByRole("button", { name: "Run CRDT merge" })).toBeEnabled();
    });

    test("CRDT demo describes the frf_crdt::apply_delta function", async ({ page }) => {
        await page.goto("/#demo/signaling");
        await expect(page.getByText(/frf_crdt::apply_delta/)).toBeVisible();
    });
});

test.describe("Layer 2: CRDT apply_delta WASM round-trip (requires WASM build)", () => {
    test.skip(skipIntegration, "Set SKIP_INTEGRATION=false and WASM_AVAILABLE=1 to run Layer 2 CRDT tests");

    test("clicking Run CRDT merge shows a result or error (not stuck loading)", async ({ page }) => {
        await page.goto("/#demo/signaling");
        const btn = page.getByRole("button", { name: "Run CRDT merge" });
        await expect(btn).toBeEnabled();

        await btn.click();

        // After click, button should either show result or show an error.
        // It must NOT stay in idle state indefinitely (loading ends with output).
        await expect(
            page.locator("[data-testid='crdt-result']").or(page.getByRole("alert")),
        ).toBeVisible({ timeout: 15_000 });
    });

    test("Run CRDT merge with WASM available shows byte-length result", async ({ page }) => {
        await page.goto("/#demo/signaling");
        const btn = page.getByRole("button", { name: "Run CRDT merge" });
        await btn.click();

        // The result pre shows: crdt_apply_delta([], []) → [] (0 bytes)
        const resultPre = page.locator("[data-testid='crdt-result']");
        await expect(resultPre).toBeVisible({ timeout: 15_000 });

        const text = await resultPre.textContent();
        expect(text).toMatch(/crdt_apply_delta/);
        expect(text).toMatch(/bytes/);
    });
});
