import { createConnectTransport } from "@connectrpc/connect-web";
import { SpineClient } from "@prometheusags/frf-sdk";

const GATEWAY_URL = import.meta.env["VITE_GATEWAY_URL"] ?? "http://localhost:4000";

const transport = createConnectTransport({ baseUrl: GATEWAY_URL });

export const spineClient = SpineClient.create(transport);
