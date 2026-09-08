---
id: overview
title: Case studies
sidebar_label: Overview
---

Two products, neither of which is an FRF deployment today, both of which solved
problems FRF's ports exist to solve. They are here because independent
convergence is evidence, and because both teams documented their reasoning
unusually well — including where they concluded their own approach was
insufficient.

## [Prior authorization (ASO)](prior-auth.md)

A clinical workbench for spine and orthopedic prior authorization, where a
mistake in the data boundary is a disclosure of protected health information.

Read it for: a PHI boundary defined at two independent layers, a guard that was
deliberately made to fail before being trusted, and a bug where the numbers were
arithmetically correct the whole time and still told a coordinator a surgical
authorization was ready to send.

Its most transferable idea is a refusal: **an accident of the technology stack
is not a control**, even when the accident produces the safe outcome.

## [KnowMe](knowme.md)

A private personal-intelligence workspace built as a Rust core with thin
platform bindings.

Read it for: a disciplined three-lane split that treats CRDTs as a considered
choice rather than a default, a structural fail-closed privacy class, and an
inference-engine decision recorded as a *failure* ("it works on the phone you
tested and fails on the next one") rather than a preference.

## A note on what these are not

Neither product currently runs on FRF.

ASO syncs ElectricSQL directly; whether to adopt FRF's authorized shape facade
is an open question in its own architecture notes. KnowMe's repository has
**zero code references** to FRF — the connection is a ratified intent with no
implementation behind it.

Presenting either as an FRF success story would be false, and this documentation
is not willing to trade accuracy for a better narrative. What they offer is
better than a testimonial: two independent teams reaching the same conclusions
about seams, defaults, and how to state what you have not proven.
