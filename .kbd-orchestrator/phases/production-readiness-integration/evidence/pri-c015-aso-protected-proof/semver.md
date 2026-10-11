# PRI c015 semver and rollout record

## Public contract change

`frf-ports` changes the public `ASO_PROJECTION_REVISION` constant from 3 to 4.
`VerifiedClaims::aso_replica_scope` performs an exact revision match, so a
revision-3 ASO replica grant is rejected after Fabric adopts revision 4. The
revision advances because ASO's server-owned registry now grants seven exact
projections, including `annotation_types` and `annotations`, and expands the
approved `cases` projection.

## Semver classification

The workspace is currently `0.1.0`. This is a behaviorally breaking public-port
change for a pre-1.0 crate and requires the next pre-1.0 minor release rather
than a patch release. The release-signoff change must carry the resulting
workspace version and migration note; c015 does not publish packages.

## Forward rollout

ASO, Flint Gate, and Fabric use one distributed exact-match contract:

1. Build Gate and Fabric from source that declares revision 4.
2. Replace both runtime images in the same maintenance window.
3. Drain or revoke revision-3 grants and handles, then issue fresh revision-4
   grants from ASO.
4. Keep the protected browser profile unavailable until both images report
   healthy and a revision-4 request passes through Gate to Fabric.

Mixed Gate/Fabric revisions fail closed with HTTP 403. No revision downgrade or
compatibility alias is provided because accepting both revisions would let an
older projection contract outlive its privacy review.

## Rollback

Rollback is coordinated across ASO's projection registry, Gate's minted claim,
Fabric's exact-match constant, and the shape catalog. Stop new grants, drain
revision-4 handles, restore the prior four artifacts together, and issue new
revision-3 grants only after the prior catalog is active. Rolling back one
participant alone intentionally leaves the protected path unavailable.
