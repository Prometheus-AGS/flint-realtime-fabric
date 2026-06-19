/**
 * Smoke-test subscriber for the TypeScript SDK.
 *
 * Usage:
 *   FRF_GATEWAY_URL=http://localhost:4000 npx tsx tests/e2e/ts/smoke.ts
 *
 * Exits 0 when it receives at least one EventEnvelope within the timeout.
 */

import { createConnectTransport } from "@connectrpc/connect-web";
import { SpineClient } from "@prometheusags/frf-sdk";

const gatewayUrl = process.env["FRF_GATEWAY_URL"] ?? "http://localhost:4000";
const channelId = process.env["FRF_CHANNEL_ID"] ?? "00000000-0000-0000-0000-000000000001";
const TIMEOUT_MS = 15_000;

const transport = createConnectTransport({ baseUrl: gatewayUrl });
const client = SpineClient.create(transport);

const timer = setTimeout(() => {
  console.error("Timeout: no event received within 15s");
  process.exit(1);
}, TIMEOUT_MS);

console.log("Subscribed; waiting for event…");

for await (const envelope of client.subscribe({
  channelId,
  consumerId: "e2e-ts-smoke",
})) {
  clearTimeout(timer);
  console.log(`OK — received envelope id=${envelope.id} kind=${envelope.kind}`);
  process.exit(0);
}
