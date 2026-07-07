import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

/**
 * Global setup: write a Playwright storage state that seeds a dev access token
 * into localStorage. This satisfies the LoginGate (added in p16-c008) so the
 * existing app-content specs render the gated UI without each needing to log in.
 *
 * Specs that specifically test the login gate (p16-admin-gateway) start from a
 * clean context via `test.use({ storageState: { cookies: [], origins: [] } })`.
 */
export default function globalSetup(): void {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  const authDir = resolve(__dirname, "../.auth");
  const statePath = resolve(authDir, "state.json");
  mkdirSync(authDir, { recursive: true });

  const state = {
    cookies: [],
    origins: [
      {
        origin: "http://localhost:5173",
        localStorage: [{ name: "frf.accessToken", value: "e2e-dev-token" }],
      },
    ],
  };

  writeFileSync(statePath, JSON.stringify(state), "utf8");
}
