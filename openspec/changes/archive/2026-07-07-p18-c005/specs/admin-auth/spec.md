# admin-auth (delta)

## ADDED Requirements

### Requirement: The admin UI MUST handle token expiry and Unauthorized

The admin UI's token gate MUST be expiry-aware: it MUST reject an already-expired token
at login, decode `exp` to know when a token expires, log out automatically when a live
token expires, and clear the token on a gateway Unauthenticated response. The login copy
MUST be honest that this is a token-entry gate, not an interactive OIDC login (which is
deferred). The single `useAuthStore.accessToken` remains the source all consumers read.

#### Scenario: an expired token is rejected at login

- **WHEN** an operator submits a JWT whose `exp` is in the past
- **THEN** login returns false and the gate shows an error instead of authenticating

#### Scenario: a live token triggers logout at expiry

- **WHEN** a stored token reaches its `exp`
- **THEN** the token is cleared and the app returns to the login gate

#### Scenario: a gateway Unauthenticated response clears the token

- **WHEN** a gateway call returns Unauthenticated
- **THEN** the interceptor clears the stored token so the UI re-prompts for login
