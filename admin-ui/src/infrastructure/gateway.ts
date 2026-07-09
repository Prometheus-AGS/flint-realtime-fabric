import { createConnectTransport } from "@connectrpc/connect-web";
import { Code, ConnectError, type Interceptor } from "@connectrpc/connect";
import { SpineClient } from "@prometheusags/frf-sdk";
import { useAuthStore } from "../features/auth/stores/authStore.js";
import { handleUnauthorized } from "../features/auth/services/authService.js";

// The gateway serves Connect / gRPC-web on its gRPC port (GRPC_PORT, default 9090),
// NOT the Axum HTTP port. Direct dev gateway → :9090; the compose stack maps the
// gRPC port to the host as 29090, so set VITE_GATEWAY_URL=http://localhost:29090
// when running the UI against compose.
const GATEWAY_URL =
  import.meta.env["VITE_GATEWAY_URL"] ?? "http://localhost:9090";

/**
 * Attaches the current access token (from the auth store) as an
 * `Authorization: Bearer` header on every outbound Connect/gRPC-web call.
 * The gateway verifies this JWT at its boundary. Without it, a secured gateway
 * rejects the request — so this interceptor is what makes authenticated calls
 * possible once the user has logged in.
 */
const authInterceptor: Interceptor = (next) => async (req) => {
  const token = useAuthStore.getState().accessToken;
  if (token) {
    req.header.set("Authorization", `Bearer ${token}`);
  }
  try {
    return await next(req);
  } catch (err) {
    // The gateway rejected the credential (expired/invalid): clear it so the UI drops
    // back to the login gate instead of retrying a doomed request.
    if (err instanceof ConnectError && err.code === Code.Unauthenticated) {
      handleUnauthorized();
    }
    throw err;
  }
};

const transport = createConnectTransport({
  baseUrl: GATEWAY_URL,
  interceptors: [authInterceptor],
});

export const spineClient = SpineClient.create(transport);
