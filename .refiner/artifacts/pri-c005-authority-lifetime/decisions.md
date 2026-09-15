# Refinement decisions — pri-c005-authority-lifetime

- Terminate generic subscriptions on their verified token deadline with an
  independent timer, including idle streams.
- Recheck the channel relation and event object relation before every protected
  event; fail closed on authorization transport errors.
- Keep Keto decisions for one second and fence in-flight cache refills with an
  invalidation generation.
- Treat the selected Keto build's HTTP 403 as a denied check and use its
  `/admin/relation-tuples` mutation path.
- Refresh JWKS after at most two seconds and serialize refreshes so an older
  response cannot overwrite a newer key set.
- Pin Cedar exactly to 4.11.2 as required by c003 and retain the 4.12 migration
  decision for a separately reviewed compatibility change.
- Use a detached Gate worktree at an exact revision and digest-pinned Postgres
  and Keto containers for the cross-repository acceptance run.
- Compile the Fabric integration test before starting owned services or minting
  credentials, and generate a new Postgres password for every run.
- Serialize Keto deletion parameters through reqwest so tuple identifiers and
  subjects are percent encoded correctly.
- Record the public JWKS cache alias-to-struct replacement as a pre-1.0 breaking
  source change requiring at least a minor version and migration notes in c023.
