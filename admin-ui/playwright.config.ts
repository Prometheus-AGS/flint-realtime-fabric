import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env["CI"],
  retries: process.env["CI"] ? 2 : 0,
  reporter: "html",
  // Seed a dev access token into localStorage so the LoginGate (p16-c008) is
  // satisfied for the app-content specs. Gate-specific specs opt out per-test.
  globalSetup: "./e2e/support/global-setup.ts",
  use: {
    baseURL: "http://localhost:5173",
    trace: "on-first-retry",
    storageState: "./e2e/.auth/state.json",
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
  ],
  webServer: {
    command: "pnpm dev",
    url: "http://localhost:5173",
    reuseExistingServer: !process.env["CI"],
    timeout: 30_000,
  },
});
