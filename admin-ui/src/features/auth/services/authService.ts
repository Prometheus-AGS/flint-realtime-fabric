import { useAuthStore } from "../stores/authStore.js";

const STORAGE_KEY = "frf.accessToken";

/**
 * Auth service — the only layer that touches token persistence.
 *
 * The admin UI does NOT perform an interactive OIDC redirect flow: flint-gate exposes no
 * browser-facing authorization-code/login endpoint (only token-exchange /
 * client_credentials), and no IdP (Kratos/Hydra) is deployed. So this is a **token-entry
 * gate** — an operator supplies a JWT minted by flint-gate — hardened here with expiry
 * awareness. A full OIDC login flow is deferred until an IdP + flint-gate login endpoint
 * exist (see docs/SECURITY.md §6).
 *
 * Security note: the token lives in `localStorage`, which is readable by any script on the
 * origin (an XSS foothold could exfiltrate it). The mitigating properties here are that the
 * token is short-lived (the gateway enforces `exp`) and this UI logs out the moment it sees
 * an expired token or a 401. A stronger posture (refresh-token-in-memory + short-lived
 * access token) requires a refresh mechanism flint-gate does not yet offer.
 */

/** Seconds of clock-skew tolerance when judging expiry. */
const EXPIRY_SKEW_SECONDS = 30;

/**
 * Decode a JWT's `exp` claim (seconds since epoch) without verifying the signature — the
 * gateway is the authority on validity; the UI only needs `exp` to avoid sending a token
 * it already knows is expired. Returns `null` when the token is malformed or has no `exp`.
 */
export function decodeJwtExp(token: string): number | null {
  const parts = token.split(".");
  if (parts.length !== 3) {
    return null;
  }
  try {
    // base64url → base64, then decode the payload.
    const payloadPart = parts[1];
    if (payloadPart === undefined) {
      return null;
    }
    const base64 = payloadPart.replace(/-/g, "+").replace(/_/g, "/");
    const json = atob(base64);
    const payload = JSON.parse(json) as unknown;
    if (
      typeof payload === "object" &&
      payload !== null &&
      "exp" in payload &&
      typeof (payload as { exp: unknown }).exp === "number"
    ) {
      return (payload as { exp: number }).exp;
    }
    return null;
  } catch {
    return null;
  }
}

/** The token's expiry as a `Date`, or `null` if it has no decodable `exp`. */
export function expiresAt(token: string): Date | null {
  const exp = decodeJwtExp(token);
  return exp === null ? null : new Date(exp * 1000);
}

/** True when the token's `exp` is in the past (with a small skew allowance). */
export function isExpired(token: string): boolean {
  const exp = decodeJwtExp(token);
  if (exp === null) {
    return false; // no exp → can't prove expiry; let the gateway decide
  }
  const nowSeconds = Date.now() / 1000;
  return exp + EXPIRY_SKEW_SECONDS < nowSeconds;
}

/**
 * Persist a token and populate the auth store. Rejects an already-expired token so the
 * UI never enters an authenticated state with a dead credential. Returns true on success.
 */
export function setToken(token: string): boolean {
  const trimmed = token.trim();
  if (trimmed.length === 0 || isExpired(trimmed)) {
    return false;
  }
  localStorage.setItem(STORAGE_KEY, trimmed);
  useAuthStore.getState().setAccessToken(trimmed);
  return true;
}

/** Clear the token from storage and the store (logout). */
export function clearToken(): void {
  localStorage.removeItem(STORAGE_KEY);
  useAuthStore.getState().setAccessToken(null);
}

/**
 * Handle a gateway `Unauthenticated` (401 / gRPC UNAUTHENTICATED) response: the stored
 * token is no longer accepted, so clear it and drop the UI back to the login gate.
 */
export function handleUnauthorized(): void {
  clearToken();
}

/**
 * Restore a persisted token into the store on app boot. A stored token that is already
 * expired is discarded rather than restored. Returns true if a valid token was restored.
 */
export function restoreToken(): boolean {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && stored.length > 0 && !isExpired(stored)) {
    useAuthStore.getState().setAccessToken(stored);
    return true;
  }
  // Expired or absent: make sure nothing stale lingers.
  if (stored) {
    localStorage.removeItem(STORAGE_KEY);
  }
  return false;
}
