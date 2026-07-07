import { useAuthStore } from "../stores/authStore.js";
import { clearToken, setToken } from "../services/authService.js";

interface UseAuth {
  /** The current access token, or null when logged out. */
  accessToken: string | null;
  /** True when a token is present. */
  isAuthenticated: boolean;
  /** Store a token (login). */
  login: (token: string) => void;
  /** Clear the token (logout). */
  logout: () => void;
}

/**
 * Coordinates the auth UI with the store and service. Components call this hook;
 * they never touch the store or service directly.
 */
export function useAuth(): UseAuth {
  const accessToken = useAuthStore((s) => s.accessToken);

  return {
    accessToken,
    isAuthenticated: accessToken !== null,
    login: setToken,
    logout: clearToken,
  };
}
