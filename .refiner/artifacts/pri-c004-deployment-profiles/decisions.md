# Refinement decisions — pri-c004-deployment-profiles

- Keep `full` and `shape-only` independent down to images, dependencies,
  readiness, network exposure and public routes.
- Disable media, agent and admin lanes by default and mount their HTTP routes
  only after explicit opt-in.
- Remove Iggy entirely from `shape-only`; use a rejecting broker at composition
  seams so internal calls cannot cross that authority boundary.
- Probe Gate's public RFC 7517 `/.well-known/jwks.json` endpoint and require a
  non-empty key set, not only HTTP success, before readiness.
- Seed Gate with a validated asymmetric private/public key pair before starting
  the runtime; reject development providers and symmetric signing algorithms.
- Treat CDC as a required full-profile readiness component. Readiness becomes
  true only after logical replication starts and false whenever the task exits.
- Use a pinned Nginx TLS edge and expose no backend host ports. Retain a
  digest-required renderer rather than inventing release image identities.
- Make the SSR renderer validate the TLS chain and key pair, then emit the exact
  named Kubernetes TLS Secret consumed by its Ingress.
- Record the additive public gateway configuration as a pre-1.0 breaking source
  change for release version selection at c023.
