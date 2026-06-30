# p14-c001 — Add dev HMAC JWKS to `deploy/oathkeeper/`

> Phase: phase-14-stage10-dind-live-triage · Priority: CRITICAL

## Problem

`deploy/oathkeeper/config.yml` references `file:///etc/config/oathkeeper/jwks.json` for the JWT authenticator, but `deploy/oathkeeper/jwks.json` does not exist. Oathkeeper crashes at startup because it cannot load the JWKS — making the entire compose stack non-functional.

## Solution

Create `deploy/oathkeeper/jwks.json` containing a valid dev HMAC JWKS.

The HMAC key is for the dev compose stack only. It is a well-known value, not a production secret.

```json
{
  "keys": [
    {
      "kty": "oct",
      "use": "sig",
      "kid": "dev-hmac-key-1",
      "k": "c2VjcmV0LWRldi1rZXktZm9yLXN0YWdlLTEwLWludGVncmF0aW9uLXRlc3Rpbmctb25seQ",
      "alg": "HS256"
    }
  ]
}
```

The `k` value is the base64url encoding of `secret-dev-key-for-stage-10-integration-testing-only` — clearly documenting its dev-only nature.

## Files Changed

- `deploy/oathkeeper/jwks.json` — NEW FILE

## Acceptance Criteria

- [ ] `deploy/oathkeeper/jwks.json` exists
- [ ] File contains a valid JWK Set with at least one `oct` key
- [ ] Oathkeeper service starts without crashing in `docker compose up`
- [ ] `GET http://gateway:8080/healthz` returns 200 through oathkeeper proxy
