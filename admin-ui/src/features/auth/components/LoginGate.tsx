import { useState } from "react";
import type { ReactNode } from "react";
import { useAuth } from "../hooks/useAuth.js";

interface LoginGateProps {
  children: ReactNode;
}

/**
 * Gates the app behind a token. When no token is present, renders a minimal
 * login form; once a token is supplied it is stored (and attached to every
 * gateway call by the transport interceptor) and the children render.
 *
 * This is a token-entry gate, not a full OIDC redirect flow — see authService.
 */
export function LoginGate({ children }: LoginGateProps): React.JSX.Element {
  const { isAuthenticated, login } = useAuth();
  const [value, setValue] = useState("");

  if (isAuthenticated) {
    return <>{children}</>;
  }

  const handleSubmit = (e: React.FormEvent): void => {
    e.preventDefault();
    login(value);
  };

  return (
    <main
      style={{
        minHeight: "100vh",
        display: "grid",
        placeItems: "center",
        padding: "2rem",
      }}
      aria-labelledby="login-heading"
    >
      <form
        onSubmit={handleSubmit}
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "0.75rem",
          width: "min(28rem, 100%)",
        }}
      >
        <h1 id="login-heading" style={{ margin: 0, fontSize: "1.25rem" }}>
          Sign in to Flint Realtime Fabric
        </h1>
        <p style={{ margin: 0, color: "var(--muted, #667)", fontSize: "0.875rem" }}>
          Paste an access token (JWT) issued by flint-gate. It is attached as a
          Bearer credential to every gateway request.
        </p>
        <label htmlFor="token-input" style={{ fontSize: "0.8125rem", fontWeight: 600 }}>
          Access token
        </label>
        <input
          id="token-input"
          type="password"
          autoComplete="off"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="eyJhbGciOi…"
          style={{ padding: "0.5rem 0.75rem", fontFamily: "monospace" }}
        />
        <button type="submit" disabled={value.trim().length === 0}>
          Continue
        </button>
      </form>
    </main>
  );
}
