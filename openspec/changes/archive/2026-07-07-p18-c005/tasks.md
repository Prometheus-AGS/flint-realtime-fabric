# Tasks — p18-c005

- [x] authService: decode JWT exp; expose isExpired / expiresAt
- [x] useAuth / authStore: warn + logout on expiry; trigger re-auth (clear + prompt) on gateway 401
- [x] LoginGate: honest copy — a token-entry gate, not OIDC; note full login is deferred
- [x] Secure-storage guidance (comment/doc): localStorage XSS caveat; short-lived-token pattern
- [x] Keep useAuthStore.accessToken populated so all 4 consumers (interceptor, WS, 2 agent streams) still work
- [x] No `any`; typecheck + lint clean
