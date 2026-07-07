import { useAuthStore } from "../stores/authStore.js";

const STORAGE_KEY = "frf.accessToken";

/**
 * Auth service — the only layer that touches token persistence.
 *
 * The admin UI does not yet perform an interactive OIDC redirect flow against
 * flint-gate; that requires flint-gate's login endpoints and is tracked
 * separately. For now a token is supplied directly (e.g. a dev token minted by
 * flint-gate, or an operator-provided JWT) and persisted so calls survive a
 * reload. When flint-gate login lands, `login()` becomes the redirect handler.
 */

/** Persist a token and populate the auth store. */
export function setToken(token: string): void {
  const trimmed = token.trim();
  if (trimmed.length === 0) {
    return;
  }
  localStorage.setItem(STORAGE_KEY, trimmed);
  useAuthStore.getState().setAccessToken(trimmed);
}

/** Clear the token from storage and the store (logout). */
export function clearToken(): void {
  localStorage.removeItem(STORAGE_KEY);
  useAuthStore.getState().setAccessToken(null);
}

/**
 * Restore a persisted token into the store on app boot. Call once at startup.
 * Returns true if a token was restored.
 */
export function restoreToken(): boolean {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && stored.length > 0) {
    useAuthStore.getState().setAccessToken(stored);
    return true;
  }
  return false;
}
