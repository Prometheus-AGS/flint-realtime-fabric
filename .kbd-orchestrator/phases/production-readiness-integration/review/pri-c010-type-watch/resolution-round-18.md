# c010 adversarial review round 18 resolution

## Critical: filtered completion used broker beginning instead of retained boundary

Disposition: fixed. Snapshot authorization now tracks the earliest retained,
in-scope broker offset withheld by object authorization. Its completion token
stores the immediately preceding offset, or no offset only when the withheld
row is broker offset zero. If no in-scope row was withheld, completion retains
the atomic projection cursor as before.

The grant/resume regression now places the withheld row at offset 5 and makes
every earlier offset unavailable. Snapshot delivery subscribes after the atomic
cursor at offset 6; after the grant, resumption succeeds only by decoding a
completion boundary of offset 4 and requesting offset 5. A `None` checkpoint
would request broker beginning and produce `HistoryExpired`, so the regression
distinguishes the corrected boundary from the previous implementation.
