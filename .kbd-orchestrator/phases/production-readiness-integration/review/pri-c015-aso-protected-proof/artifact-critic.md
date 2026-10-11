# Artifact critic — pri-c015-aso-protected-proof

## Verdict: FAIL

The frozen candidate cannot close c015. Direct-backend isolation is false for the
selected host-browser topology, and the live Gate/FRF runtime images are not
bound to the source revisions claimed by the receipt. Two other required
behaviors are demonstrated only by mocked/component tests rather than across the
selected protected browser topology.

Independent integrity checks did succeed: all 14 artifacts inventoried by
`receipt.json` match their recorded byte counts and SHA-256 values; every file in
`source-manifest.json` and all three canonical aggregate hashes recompute
exactly; the separately recorded task-3/task-4 source hashes also match current
files. The browser-only `shape-only` selection agrees with c003
`release-contract.md:19-41`, and excluding native clients is correct. The two
c015-created ASO files are 498 and 89 lines; the recorded untouched 524/659-line
files remain unchanged. Campaign before/after status and diff hashes match, so I
found no evidence that unrelated dirty work changed during the run.

## BLOCKING

### B1. The selected browser can bypass Gate and read Electric directly

`docker-compose.ra05.yaml:10-14` publishes Electric on `127.0.0.1:3000`, while
the selected browser is launched on the same host from
`scripts/test-ra11c-browser-memory.py:142-191`. The isolation test substitutes a
Docker `topology-probe` attached only to the internal client network
(`scripts/test-authorized-shape-topology.py:92-133,176-198`) and declares success
when that container cannot reach `host.docker.internal`
(`scripts/test-authorized-shape-topology.py:241-264`). That is not the network
position of the qualified browser.

I reproduced the bypass against the exact campaign images still running
(`Electric sha256:9644bb...`, Gate `sha256:5ad074...`, FRF `sha256:b36243...`):

```text
curl -H 'Origin: http://127.0.0.1:4173' \
  'http://127.0.0.1:3000/v1/shape?table=aso.cases&offset=-1'
=> HTTP 200
=> access-control-allow-origin: http://127.0.0.1:4173
=> electric-schema includes member_id, patient_id, payer_id, data, and other
   columns outside the approved cases projection
```

This violates c003 `release-contract.md:24,71-72,125`: the browser can choose the
table and bypass Gate's session, practice, catalog, and projection checks. It
also disproves `acceptance-b.md:32-42` and the direct-backend/egress part of the
OpenSpec scenario (`spec.md:13-16`).

Required change: remove the host-reachable Electric surface from the qualified
topology (or isolate the actual browser from it), then add a real browser-origin
negative probe that attempts direct FRF/Electric access and fails while the
Gate path succeeds. Rerun and refreeze the full campaign.

### B2. “Current-source” runtime execution is not established

The coordinator hashes the three checkouts, but the live campaign only stops
and starts existing containers. `scripts/test-ra11a-sync-conformance.py:309-331`
runs `compose up -d` without `--build`; `campaign.json:1955-2023` records the Gate
and FRF containers as pre-existing and `campaign.json:2043-2106` records the same
image digests afterward. The two local images have only Compose labels and no
source revision/content provenance. Thus the manifest proves what was in the
worktrees, not what binaries ran inside Gate and FRF.

Required change: build immutable Gate/FRF images from the frozen source inputs,
record build-context/source aggregates and revision labels or attestations,
start containers by exact image digest, and verify the running image IDs against
those source-bound build receipts before executing acceptance.

## HIGH

### H1. Interrupted-body acceptance is only a mocked Node transport test

The baseline explicitly says interruption remained to be joined across the
selected browser topology (`baseline.md:53-57`). The only added proof is a Node
test (`protected-replica-boundaries.integration.test.ts:1,20-28`) whose mocked
`fetch` returns a synthetic errored `ReadableStream`
(`protected-replica-boundaries.integration.test.ts:46-60`). It does not interrupt
a live Gate/FRF/Electric response, execute in the selected browser, or observe
the real SQL/checkpoint state. Nevertheless `acceptance-b.md:5-15` treats it as
acceptance for that topology.

Required change: inject a body truncation/reset in the live protected path used
by the actual browser, assert no partial SQL row/checkpoint/PEM publication, then
prove an authorized cold retry commits the complete revision.

### H2. Logout/practice-switch graph clearing is not exercised in the mounted topology

`task-3-focused-tests.json:4-18,46` is a separate 26-test component/unit run over
the session provider, access boundary, graph-session manager, and quiescence
seam. The mounted PGlite-to-PEM test
`replica-mounted.integration.test.ts:50-448` never logs out, changes practice, or
asserts that a real populated graph is cleared. The evidence therefore proves
UI fencing and disposal seams in isolation, not the required outcome in
`spec.md:8-11` that logout/practice switch fences stale owners and clears the old
graph for the selected topology.

Required change: in the mounted browser campaign, populate SQL and PEM, perform
real logout and a separately authorized practice switch, then assert immediate
protected-view closure, owner cancellation/drain, absence of the old graph, no
post-fence row/checkpoint publication, and clean replacement-scope hydration.

## MEDIUM

### M1. The final source manifest omits acceptance-critical inputs

The ASO aggregate includes `graph-provider.tsx` and `web/src/shared/sync`, but not
the session provider and two provider tests used by task 3
(`task-3-focused-tests.json:14-38`). The Fabric aggregate includes only
`shape_revocation.rs` among integration tests
(`source-manifest.json:282-338`), while the claimed 10 timing tests also depend
on `shape_lease_timing.rs` and `tests/support/shape_timing.rs`
(`task-4-revocation-timing.json:17-27`). The manifest also omits the local pinned
PEM tarballs loaded by the browser tests. Secondary receipts record hashes for
some of these files, but `sourceMatchesCampaign: true` and the three advertised
aggregates are not a complete binding of the accepted proof.

Required change: include every test, helper, production dependency, local PEM
package artifact, Compose/Dockerfile/build input, and generated configuration
that can affect acceptance in the final manifest and aggregate computation.

### M2. Generic-writer exclusion is a lexical heuristic, not behavioral proof

`scripts/test-pri-c015-protected-replica.py:72-78,176-203` searches local ASO
source for six literal symbol spellings and checks that the provider text
contains two names. It does not inspect packaged dependencies, aliases/dynamic
access, resolved feature configuration, or runtime subscriptions/writes. Zero
regex matches cannot establish the OpenSpec claim that no competing generic
watch writer handles the clinical rows.

Required change: bind the complete resolved production dependency closure and
add a runtime assertion/trace showing the sole clinical writer and absence of a
generic watch subscription or second SQL/checkpoint writer during the campaign.

## LOW

### L1. Successful cleanup leaves uninventoried continuation state

`task-4-mounted-materializer.json.state.json` and `.progress` remain beside the
primary evidence but are absent from `receipt.json`'s artifact inventory. The
state file contains case/checkpoint identifiers and Electric handles. They are
run scratch state, not historical attempts.

Required change: delete these files after successful continuation, or redact and
inventory them explicitly if they are intended evidence.
