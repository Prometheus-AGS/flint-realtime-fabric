# media-e2e

## ADDED Requirements

### Requirement: The decode runner uses a pre-built gateway image, never an in-run build
The decode runner SHALL use a pre-built gateway image and MUST NOT rebuild it during the decode run;
when the image is absent it fails fast with the out-of-band build command, so a single run cannot OOM
the host by compiling the gateway image concurrently with the stack.

#### Scenario: Pre-built image present
- **WHEN** the gateway image exists
- **THEN** the runner brings up the stack with `--no-build` and does not rebuild.

#### Scenario: Image absent
- **WHEN** the gateway image is absent (and `PREBUILD_GATEWAY` is unset)
- **THEN** the runner exits non-zero with the one-time build command, not a silent in-run rebuild.
