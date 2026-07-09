# Tasks — p31-c001-prebuilt-image-runner

- [x] 1. run-media-decode.sh: presence-check the implicit gateway image; present→skip build; absent→fail fast with the out-of-band build hint; PREBUILD_GATEWAY=1 escape hatch.
- [x] 2. Document the one-time build + colima start --memory prerequisite in the runner header; bash -n / shellcheck clean.
