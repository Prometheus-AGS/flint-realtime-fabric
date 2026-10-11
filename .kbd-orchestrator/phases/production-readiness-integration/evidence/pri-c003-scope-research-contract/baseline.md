# Baseline — pri-c003-scope-research-contract

Captured: 2026-09-15

## Fabric source

- Branch: `codex/production-readiness-integration`
- HEAD: `52ab9b3874abbe35bbca73720d85578f3db70943`
- `analysis.md`: `676746bcff906c0e3055199c00e058e71d1d124eecfb7302ca8908fe9288d9c8`
- `decision-log.md`: `33c20a88110e907b5d73f66644f8847f196d6e3fcf712525e35fd439ef622b14`
- `library-candidates.json`: `d847bf7bd2c23eba8fbc94d8f0189b2c73f987d6d29e0816809aa5abc312d75c`

The three pre-existing shape edits remain outside this change and retain the
SHA-256 values recorded in `entry-state.json`.

## Related repository revisions

| Repository | Revision | Worktree status hash |
|---|---|---|
| flint-gate | `97d65437c937a285fb000f5e42fddc8af4ff9934` | empty status (`e3b0c442…b855`) |
| flint-forge | `dc313be3a044c65b05d845a9c34350bf5ca3ca3e` | empty status (`e3b0c442…b855`) |
| prometheus-entity-management | `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565` | empty status (`e3b0c442…b855`) |
| prior-auth | `d95242542060d1e19a6efe9143f57c015d6162f4` | dirty status hash `ef91609a4f7f443246c510f2bb50547b4fe9e652fb9afd5c32d6d2f4dfe4f70c` |

Forge advanced from the phase-entry revision while remaining clean; current
source is authoritative for research. The ASO checkout has extensive active
work and is read-only for c003. Later ASO changes must use an isolated worktree.

## Dependency receipts

c003 has no implementation dependency. Earlier phase receipts are context:

- c001 verification receipt SHA-256:
  `252cee314685c9ce6d4be2c5d68a56eb79992cc87ecd354b0e61abdb727e574b`
- c002 verification receipt SHA-256:
  `4409fe7fa6ca1db3b01091ea8160fb4317a8102b8fd1e5074912177185716622`
- c002 final independent findings SHA-256:
  `9a0f903cd9420d320dceed8316d2e6787e58d41e5ed51ed3b74fe8a5aff8c3c5`

No historical runtime receipt is accepted as c003 qualification evidence.
