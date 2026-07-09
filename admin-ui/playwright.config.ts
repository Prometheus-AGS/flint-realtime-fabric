import { defineConfig, devices } from "@playwright/test";

// DECODE_ONLY (p31-c002): the media-decode spec talks to the sovereign gateway
// (`gateway:8080`), NOT the admin-ui Vite dev server — and when run in the Playwright
// container `pnpm` is not on PATH, so `webServer: pnpm dev` aborts with exit 127 before the
// test runs. In decode-only mode we skip the dev server, its `baseURL`, the dev-token
// `globalSetup`, and the `storageState` auth file — none apply to the gateway-driven spec.
const decodeOnly = process.env["DECODE_ONLY"] === "1";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env["CI"],
  retries: process.env["CI"] ? 2 : 0,
  reporter: "html",
  // Seed a dev access token into localStorage so the LoginGate (p16-c008) is
  // satisfied for the app-content specs. Gate-specific specs opt out per-test.
  ...(decodeOnly ? {} : { globalSetup: "./e2e/support/global-setup.ts" }),
  use: {
    trace: "on-first-retry",
    ...(decodeOnly
      ? {}
      : {
          baseURL: "http://localhost:5173",
          storageState: "./e2e/.auth/state.json",
        }),
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  ...(decodeOnly
    ? {}
    : {
        webServer: {
          command: "pnpm dev",
          url: "http://localhost:5173",
          reuseExistingServer: !process.env["CI"],
          timeout: 30_000,
        },
      }),
});
