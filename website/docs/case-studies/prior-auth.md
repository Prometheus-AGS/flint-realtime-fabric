---
id: prior-auth
title: Prior authorization — a PHI boundary under real constraints
sidebar_label: Prior authorization (ASO)
---

The Prior Authorization Workbench is a clinical application for spine and
orthopedic surgery. It is the most demanding consumer of FRF's design ideas,
because a mistake in its data boundary is not a bug report — it is a disclosure
of protected health information.

This page is the most useful case study on this site for one reason: **the team
repeatedly refused to accept accidental safety as designed safety**, and wrote
down why each time.

:::note Relationship to FRF
ASO does not currently use FRF's shape facade. Whether to adopt it is an open
question recorded in the project's own architecture notes. What follows is a
study of the same problems FRF's [authorization model](../theory/authorization.md)
addresses, solved independently under clinical constraints — including places
where ASO's team concluded their own approach was **not sufficient**.
:::

## The clinical problem

A surgeon decides an operation is necessary; an insurer decides whether to pay
for it. Between those decisions sits a document, and most of the work is
assembling the evidence it stands on.

Three failure modes stack:

**Routing.** The entity that decides is often not the entity on the insurance
card — several payers delegate musculoskeletal decisions to specialised vendors.
A submission sent to the payer instead of the delegated vendor does not get
denied. It *sits*, and the statutory clock never starts.

**Documentation.** Criteria converge on a small set of documentable facts —
graded imaging correlated to exam findings, weeks of supervised conservative
care with dates, instability measured in millimetres and degrees, functional
scores — and diverge exactly where a practice loses cases.

**Appeals.** Per a 2024 AMA survey, **81.7% of appealed prior-authorization
denials are overturned** in whole or in part. That number is the business case:
most initial denials are not clinically robust, so a practice that appeals
systematically recovers most of what it loses — but only if appealing becomes
cheap enough to always be rational.

## An architectural reversal worth studying

The project originally specified a local-first desktop application with PHI on
one machine. The team reversed it, and the reasoning generalises:

> [the original posture] does not survive contact with this workflow

A coordinator, a scheduler, a biller, two surgeons and a physician assistant all
touch the same case. They need shared state and a shared work queue. Pure
local-first autonomy is the wrong shape for collaborative clinical work.

So "local-first" here means **a local replica of a server-authoritative store**,
not peer-to-peer autonomy. That distinction shapes everything below — and it is
the same posture FRF's [shape facade](../guides/local-first.md) assumes.

## The PHI boundary

### Layered by construction

The boundary is defined at two independent layers: what the local database can
physically hold, and what is requested on the wire. Neither alone is the
control.

The schema file opens by quoting the server schema's reasoning:

> "Embeddings of clinical text ARE PHI. Text can be reconstructed from an
> embedding by inversion (IEEE S&P 2023), so an embedding fails both Safe Harbor
> and Expert Determination."
>
> The same reasoning applies one step earlier: the clinical text itself is PHI
> before anyone embeds it.

Then comes the sentence that makes this codebase worth studying:

> PGlite has no pgvector, so an embedding table could not sync even by accident.
> **That coincidence is not a control.** The control is this file and the test
> that fails when a sixth table appears.

An accident of the technology stack produced the safe outcome. The team refused
to count it, because an accident can be reversed by an upgrade nobody reviewed
as a security change.

### Exclusions as assertable data

Omitted columns are a data structure, not a comment:

```ts
/**
 * Columns deliberately omitted, and why.
 *
 * Kept as data rather than a comment so the exclusion test can assert it and
 * a reviewer can diff it. Removing an entry here is a decision about PHI, not
 * a cleanup.
 */
export const OMITTED_COLUMNS: Record<string, { column: string; reason: string }[]> = {
  case_evidence: [
    {
      column: "rationale",
      reason:
        "Free clinical text written by a clinician arguing a gap. PHI. The timeline renders the state and the action, not the argument.",
    },
  ],
```

Seven columns are excluded across three tables. The most instructive is an
untyped `jsonb` column, excluded because *"cannot be shown to be PHI-free, so it
is excluded."* The default is **exclude unless provably safe** — not include
unless provably unsafe.

A meta-test asserts the reasons are real, not placeholders:

```ts
it("every omission carries a stated reason", () => {
  for (const omissions of Object.values(OMITTED_COLUMNS)) {
    for (const { column, reason } of omissions) {
      expect(reason.length, `${column} has no reason`).toBeGreaterThan(20);
    }
  }
});
```

And the two worst columns are asserted by name, so a rename cannot quietly
reintroduce them:

```ts
it("never stores verbatim chart text — the two columns that would", () => {
  // evidence_citations.quote and case_evidence.rationale are the plainest
  // PHI in the evidence path. Asserted by name so a rename does not silently
  // reintroduce them under a different label.
  expect(PGLITE_SCHEMA_SQL).not.toMatch(/^\s*quote\s/mi);
  expect(PGLITE_SCHEMA_SQL).not.toMatch(/^\s*rationale\s/mi);
});
```

### A measurement with a control arm

The wire-level projection was verified empirically, not assumed:

> **Base tables, not views.** Measured against a live stack:
>
>     GET /v1/shape?table=aso.sync_cases  -> 400 "does not exist"
>     GET /v1/shape?table=aso.cases       -> 200 snapshot-end    [control]

The `[control]` annotation is the detail worth copying. Without the second line,
the first proves only that *something* failed. With it, the finding is specific:
Electric replicates from the logical replication stream, a view emits no WAL of
its own, so a view can never join a publication. Structural, not a configuration
gap.

A separate canary-row test confirmed an unprojected shape shipped `author_name`,
`patient_id`, and `storage_uri` on the wire, while the projected shape returned
the same row carrying only the listed columns.

### The guard was proved to fail

The exclusion test was executed *and deliberately sabotaged* to confirm it
catches the thing it claims to catch, then reverted. The project's standard says
why:

> A guard that has never failed is a hypothesis.

This is the highest standard of evidence anywhere in the three repositories this
site draws on.

## The most important caveat on this page

Everything above describes a real, tested, deliberate boundary. **The project
itself has formally downgraded it** from "the control" to "useful validation."

ADR-007, the column-exclusion decision, now carries a superseding banner. Its
successor states:

> Gate and the facade derive allowed rows, columns and practice scope
> server-side from verified identity. Every continuation is authorized. **A
> client predicate or tenant-scoped adapter is useful validation but cannot
> enforce access against a modified client.**
>
> **IDs and document names can still be sensitive; neither a small table set nor
> extension availability proves that a replica contains no PHI.**

That is the same skepticism the codebase applied to PGlite's missing pgvector,
turned on its own client-side boundary. A client cannot enforce access against a
modified client. The authoritative control must be server-side — which is
exactly what FRF's [shape facade](../guides/local-first.md) is for, and which is
**not yet implemented**.

Do not read this case study as "client-side column exclusion is sufficient." The
team does not.

## The evidence state model

### Three states, never two

```ts
// Three evidence states, never two.
//
// `void` is not a weak `gap`. A gap is a chart that says no and must be
// ARGUED; a void is a chart that is silent and must be OBTAINED. They route
// work to different people, which is why the union has three members and the
// UI has three treatments.

export const EVIDENCE_STATES = ['met', 'gap', 'void'] as const;
```

| State | Meaning | Asks a human to |
|---|---|---|
| `met` | A dated source document satisfies the criterion | Cite it |
| `gap` | A document exists and contradicts or falls short | **Argue** it — surgeon |
| `void` | Nothing in the record addresses it | **Obtain** it — coordinator |

The consequence of collapsing them, stated in the decision record: *"Collapsing
them into one 'unmet' bucket sends the wrong person to do the wrong job — which
is how a case sits for three weeks waiting on a nicotine test nobody ordered."*

The union is held in four places — a database lookup table with a foreign key
(not a boolean), a Rust enum, a TypeScript union, and a Dart enum whose
unknown-value parse *throws* rather than defaulting. Because `void` is a
reserved word in Dart, the member is `voidState` while the wire value stays
`void`: *"The JSON contract is shared across three languages and does not bend
to one language's grammar."*

### The defect: zeros that were never wrong

This is the best bug story in any of these repositories, because the code was
arithmetically correct the entire time.

`countStates` seeds `{met: 0, gap: 0, void: 0}`. That is correct for a loaded
case with no matching entries — and **identical** to a case still hydrating.
Downstream, `blockedOn` falls through every `> 0` check and returns **"Ready to
draft."**

```
 * `countStates` seeds `{met: 0, gap: 0, void: 0}`, which is correct for a
 * loaded case with no entries and indistinguishable from a case still
 * hydrating. `blockedOn` falls through every `> 0` check on all-zero counts and
 * returns **"Ready to draft"** — so a partially loaded case tells a coordinator
 * it is ready. The zeros are honest; the missing distinction is what is not.
```

"The zeros are honest; the missing distinction is what is not."

Why this is a clinical-safety problem rather than a UI polish problem: "Ready to
draft" gates the next clinical step — drafting the letter of medical necessity
that goes to the payer under a physician affirmation. A mid-hydration case
claiming readiness invites a coordinator to advance a case whose chart has not
been examined. The specification calls source citation on every generated
assertion *"a False Claims Act control, not a UX flourish."* And the failure is
**quiet**: nothing errors, nothing is visibly missing.

### Why it is not a fourth state

The tempting fix — add `unknown` to the union — was rejected:

> "Not loaded" is not a fourth thing a chart can say about a criterion — it is a
> statement about the *reader*, not the evidence. Adding it to the union would
> put a UI concern into a clinical contract and break every exhaustive match in
> every language.

So the counts get wrapped instead:

```ts
export type EvidenceTally =
  | { readonly loaded: true; readonly counts: EvidenceCounts }
  | { readonly loaded: false };
```

And the consumer returns `null` rather than a placeholder string:

```ts
export function blockedOnTally(tally: EvidenceTally, gateAffirmed: boolean): string | null {
  // Not a sentence. A caller that renders this must show a loading treatment,
  // and returning null rather than a placeholder string means it cannot
  // accidentally print one.
  if (!tally.loaded) return null;
  return describeOutstanding(tally.counts, gateAffirmed);
}
```

A placeholder string would render. `null` forces the caller to decide.

### A second defect, hidden behind the first

Adversarial review found that `blockedOn` originally returned on the *first*
non-zero state:

```
  // An earlier version returned on the first non-zero state, so a case with
  // {met: 4, gap: 3, void: 1} read "1 document to obtain" and never mentioned
  // the three contradictions needing a surgeon's argument. The coordinator
  // obtains the document, the case still does not advance, and nobody can see
  // why. That is worse than ADR-003's collapse warning: the gap was not merged
  // into the void, it was HIDDEN BEHIND it.
```

The decision record warned about collapsing two states into one. The real bug
was subtler than the failure the record anticipated — a good argument for
adversarial review of artifacts, not just diffs.

## Fail-safe ordering

The rebuild sequence contains the sharpest small piece of reasoning in the sync
layer:

```
 * 1. **Bump the generation first.** If the process dies mid-rebuild, a bumped
 *    generation with stale rows still present is *safe* — every checkpoint from
 *    the old generation is now recognisably stale, so the next start rebuilds
 *    again. Clearing first and dying before the bump would leave an empty
 *    replica that still claims the old generation, and a resume would accept it.
```

Both orderings look equivalent until you ask what a crash between the two steps
leaves behind. One leaves a replica that rebuilds again; the other leaves an
empty replica that claims to be complete.

Relatedly, a rebuild **removes** rather than merges: *"a row that was deleted
server-side simply never arrives in the new snapshot, so a merge preserves it
forever."* For PHI, that is a privacy argument as much as a correctness one — a
merge silently retains revoked rows.

## Multi-tenancy: the storage key

```ts
export function graphStorageKey(session: VerifiedSession): string {
  return [
    "aso",
    `g${REPLICA_GENERATION}`,
    session.principal,
    session.practiceId,
    session.identityId,
  ].join(":");
}
```

Four components, four distinct reasons: the generation invalidates the namespace
when the schema changes shape; `principal` separates an agent acting for a
clinician from the clinician (they are different principals); `practiceId` is
the tenant boundary; `identityId` fixes the shared-workstation case.

The previous key was `aso:${practiceId}` — practice alone. Two clinicians in one
practice on a shared workstation would have shared a namespace.

An open item is recorded rather than approximated: authorization-scope revision
is *not* a component, because the session carries capabilities but no revision
counter, and hashing the capability list *"would churn the namespace on
unrelated changes."* Naming the gap beats papering over it.

## Verification status

Stated with the same discipline the rest of this site uses.

| Component | Status |
|---|---|
| PHI exclusion tests | **Executed**, and proved to fail under deliberate sabotage |
| Evidence-timeline tests | **Executed**; the state-collapse guard proved to fail |
| Wire projection, base-table-vs-view | **Measured** against a live stack, with a control |
| Storage key, session manager, chunk writer, publisher, rebuild tests | **Authored and type-checked, never executed** |
| End-to-end sync (a row flowing Postgres → Electric → replica) | **Never happened** |
| Persisted local storage | **Not enabled** — in-memory only, because "PHI at rest in the browser" is an untaken decision |
| FRF shape facade adoption | **Open question** |

## What FRF takes from this

**Refuse accidental safety.** Three times — pgvector's absence, the schema's
missing columns, and finally the client boundary itself — this team declined to
count a fortunate accident as a control. FRF's shape catalog should treat an
unlisted column as excluded by decision, not by omission.

**Say what the reader does not know.** The `EvidenceTally` distinction is not
about evidence; it is about the reader's knowledge. Any system rendering derived
state from data that arrives asynchronously needs a way to say "I do not know
yet" that is distinct from "nothing."

**Order operations for the crash.** The generation-bump ordering is correct
because of what a mid-operation failure leaves behind.

## See also

- [Local-first](../guides/local-first.md) — the FRF facade this points toward
- [Authorization](../theory/authorization.md) — server-side enforcement
- [KnowMe](knowme.md) — a structural privacy boundary reached independently
