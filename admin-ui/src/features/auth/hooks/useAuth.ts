import { useEffect } from "react";
import { useAuthStore } from "../stores/authStore.js";
import {
  clearToken,
  expiresAt,
  isExpired,
  setToken,
} from "../services/authService.js";

interface UseAuth {
  /** The current access token, or null when logged out. */
  accessToken: string | null;
  /** True when a non-expired token is present. */
  isAuthenticated: boolean;
  /** When the current token expires, or null when there is none / no `exp`. */
  tokenExpiresAt: Date | null;
  /**
   * Store a token (login). Returns false if the token is empty or already expired,
   * so the caller can surface an error instead of entering a broken authed state.
   */
  login: (token: string) => boolean;
  /** Clear the token (logout). */
  logout: () => void;
}

/**
 * Coordinates the auth UI with the store and service. Components call this hook;
 * they never touch the store or service directly.
 *
 * Also runs an expiry watchdog: when the stored token passes its `exp`, it is cleared
 * automatically so the app drops back to the login gate rather than firing requests the
 * gateway will reject.
 */
export function useAuth(): UseAuth {
  const accessToken = useAuthStore((s) => s.accessToken);

  // Expiry watchdog: log out exactly when the current token expires (and immediately if
  // it is already expired). Re-armed whenever the token changes.
  useEffect(() => {
    if (accessToken === null) {
      return;
    }
    if (isExpired(accessToken)) {
      clearToken();
      return;
    }
    const at = expiresAt(accessToken);
    if (at === null) {
      return; // no exp claim → nothing to schedule
    }
    const msUntilExpiry = at.getTime() - Date.now();
    // setTimeout caps at ~24.8 days; a token that far out effectively never expires here.
    const timer = window.setTimeout(clearToken, Math.max(0, msUntilExpiry));
    return () => {
      window.clearTimeout(timer);
    };
  }, [accessToken]);

  return {
    accessToken,
    isAuthenticated: accessToken !== null && !isExpired(accessToken),
    tokenExpiresAt: accessToken === null ? null : expiresAt(accessToken),
    login: setToken,
    logout: clearToken,
  };
}
