# c008 acceptance B — checkpoint safety and replay

Status: **PASS**

The same source-bound live fixture proves the checkpoint boundary under four
adversarial conditions:

1. A broker wrapper accepts the first event durably and then returns an injected
   failure before the CDC consumer calls `update_applied_lsn`. PostgreSQL's
   confirmed flush LSN remains unchanged.
2. Restarting the same slot replays the transaction. The stable event identity
   lets Iggy deduplicate the already durable first mutation while the second
   mutation is published. Exactly two unique mutations remain and the slot then
   advances.
3. PostgreSQL `numeric 'NaN'` poisons canonical decoding. No event is published
   and the slot does not advance beneath the transaction.
4. A post-enrollment schema change is detected by commit-boundary catalog
   revalidation. No changed-schema event is published and the slot does not
   advance.

Unit tests additionally exercise incomplete old-key data as a poison transaction
and absent old-tuple handling. The latter is accepted only when the event's
replica-identity metadata proves every entity-key and tenant-routing column is
part of PostgreSQL's unchanged identity; otherwise it poisons the transaction.
The tests also cover unchanged TOAST representation, stable event/envelope IDs
on replay, and initial Relation-frame suppression in pinned `pg_walstream 0.6.3`.

There is no acknowledged-loss window in the tested path: applied LSN changes
only after every mutation in the source commit has been accepted durably.
