# Prometheus Base Rules Set

Canonical base rules for Claude Code, Codex, OpenAI agents, Gemini CLI, Roo, Cline, Kilo Code, Librefang, and all Prometheus/UAR-compatible development agents.

These rules define how agents should reason, code, modify files, preserve architecture, and interact with human operators.

This document is the base `CLAUDE.md` and `AGENTS.md` rule source for this project. Project-specific files may add stricter requirements (see Rule 26).

---

## 1. Think Before Coding

Do not assume. Do not hide confusion. Surface tradeoffs before implementation.

Before implementing:
- State assumptions explicitly.
- If uncertain, ask.
- If multiple interpretations exist, present them.
- If a simpler approach exists, say so.
- If something is unclear, stop and ask.

---

## 2. Simplicity First

Write the minimum code that solves the problem.
- No features beyond what was requested.
- No speculative abstractions.
- No unnecessary configurability.
- No future-proofing that was not requested.
- No overengineering.

If 50 lines solves the problem, do not write 200.

---

## 3. Surgical Changes

Touch only what is necessary.
- Do not refactor unrelated code.
- Do not reformat unrelated files.
- Match existing conventions.
- Remove only artifacts created by your changes.
- Mention unrelated issues; do not fix them unless asked.

---

## 4. Goal-Driven Execution

Define success criteria first.
- Convert vague requests into testable outcomes.
- Verify completion.
- Run tests where available.
- Do not stop at implementation.
- Stop only when success criteria are satisfied.

---

## 5. Truth Over Fluency

Never prefer a confident answer over a correct answer.
- Distinguish facts from assumptions.
- Distinguish observations from conclusions.
- State uncertainty explicitly.
- Do not invent APIs, functions, files, packages, commands, or behavior.
- If something is not known, say so plainly.

---

## 6. Evidence Before Conclusions

When making claims:
- Cite evidence where available.
- Show the reasoning path.
- Explain tradeoffs.
- Explain why alternatives were rejected.
- Prefer primary sources, source code, tests, official documentation, or direct observation over guesses.

---

## 7. Preserve User Intent

Optimize for the user's actual goal.
- Do not substitute your own preferences.
- Do not silently expand scope.
- Do not silently reduce scope.
- Clarify when requirements conflict.
- Preserve the user's architectural direction unless explicitly told otherwise.

---

## 8. Minimize Irreversible Actions

Before destructive or hard-to-reverse actions:
- Confirm intent.
- Explain consequences.
- Prefer reversible approaches.
- Create rollback paths when possible.
- Never delete, overwrite, migrate, or rewrite major structures without clear authorization.

---

## 9. Maintain Architectural Consistency

Prefer consistency over novelty.
- Follow existing architecture.
- Follow existing patterns.
- Follow existing naming conventions.
- Follow existing state-management conventions.
- Avoid introducing new frameworks without justification.
- Do not create one-off architectural exceptions.

---

## 10. Keep Context Explicit

Never rely on hidden assumptions.
- State dependencies.
- State constraints.
- State limitations.
- Record decisions.
- Document important reasoning in the appropriate project file.
- Make implicit contracts explicit.

---

## 11. Architecture Before Code

Before implementation, identify:
- Affected subsystems.
- Data flow.
- Interface contracts.
- Persistence impact.
- UI impact.
- Security impact.
- Runtime impact.
- Testing strategy.

Never start coding until the architecture is understood.

---

## 12. Open Standards First

Prefer open, portable, ecosystem-agnostic standards.

Preferred standards and interfaces include:
- MCP
- OpenAI-compatible APIs
- A2A
- AG-UI
- A2UI
- HTMX
- WASM Component Model
- JSON Schema
- OpenAPI
- GraphQL where appropriate
- PostgreSQL-compatible storage
- IPFS-compatible distribution where appropriate

Avoid vendor lock-in unless explicitly required.

---

## 13. No Hidden State

Business state must live in explicit, inspectable systems.

State belongs in:
- Databases
- Event streams
- Explicit stores
- Durable queues
- Documented runtime state containers

State must not be hidden inside:
- UI components
- Untracked globals
- Implicit caches
- Framework magic
- Agent-only memory without persistence or auditability

---

## 14. Cross-Platform Parity

Any feature proposal must consider:
- Web
- Mobile
- Desktop
- Local execution
- Cloud execution
- Offline or degraded operation where relevant

Do not design features that unnecessarily trap the platform in a single runtime, framework, vendor, or deployment model.

---

## 15. Feature-Based Clean Architecture Required

All codebases shall be organized around features, domains, or bounded contexts rather than technical layers.

Preferred structure (Rust workspace):

```text
crates/
├── frf-domain/        # pure types — feature: domain capability
├── frf-ports/         # trait seams — feature: port contracts
├── frf-app/           # use-cases  — feature: business behaviors
├── frf-broker-iggy/   # one port: LogBroker
├── frf-authz-keto/    # one port: AuthzProvider
└── frf-gateway/       # interface layer
```

Rules:
- Organize by business capability first.
- Keep feature logic inside the owning crate.
- Shared code must be genuinely reusable.
- Cross-feature dependencies must be explicit and inward-pointing.
- Business logic belongs to the feature domain, not the interface layer.

---

## 16. Strict Layering Is Mandatory

Every application must enforce clear architectural boundaries.

```text
Interface (frf-gateway)
      ↓
Application (frf-app)
      ↓
Ports (frf-ports)   ←  implemented by Adapters (frf-broker-*, frf-authz-*, ...)
      ↓
Domain (frf-domain)
```

The dependency rule is absolute and points inward: `domain ← app ← infrastructure/interface`. Nothing in domain or app may import an adapter crate. This must be enforced by keeping adapter crates out of `frf-domain` and `frf-app` `[dependencies]`.

---

## 17. UI Components Must Remain Pure

UI components (React 19 / shadcn / Base UI) are responsible only for:
- Rendering
- User interaction
- Layout
- Styling
- Accessibility

UI components must not fetch data, call APIs, call services, perform business logic, manage persistence, or execute workflow logic.

---

## 18. Hooks/View Models Coordinate UI State

React hooks connect UI to stores, compose UI state, and perform presentation logic. They must not call APIs directly or contain domain business rules.

---

## 19. Stores Own Application State

Stores are the single source of truth for application state. They call services, coordinate data loading, and expose reactive state. They must not contain UI rendering logic.

---

## 20. Services Own External Communication

Services handle API calls, database access, MCP communication, agent communication, and external integrations. They must be reusable, testable, and framework-independent where possible.

---

## 21. State Changes Must Be Reactive

State changes propagate through the framework's native reactive mechanism. Avoid manual refresh calls, hidden mutable state, or direct component manipulation.

---

## 22. Dependency Versions Must Be Verified

Before introducing libraries, frameworks, SDKs, language runtimes, or build tools, verify current compatible versions:
1. Check official documentation.
2. Check official repositories.
3. Check compatibility matrices.
4. Verify against project requirements.
5. Verify against existing dependencies.

Never assume versions. Never use stale examples without verification.

---

## 23. Web Verification Before Dependency Introduction

When internet access is available, search for:
- Latest stable version.
- Known compatibility issues.
- Breaking changes.
- Migration requirements.
- Security advisories.

Priority order: official documentation → official repository → official release notes → vendor-maintained migration guides.

---

## 24. Consistency Across Languages

The architecture remains the same regardless of language.

Rust HTMX (gateway):
```text
Handler → App Use-Case → Port Trait → Adapter → External System
```

React (admin UI):
```text
Component → Hook → Store → Service → API
```

Technology changes. Architecture does not.

---

## 25. Human Override Always Exists

Every automated decision must support inspection, auditability, override, recovery, manual correction, and human escalation.

---

## 26. Repo-Level Rules Override Base Rules Only When Explicit

These are base rules. Project-specific CLAUDE.md, AGENTS.md, README.md, architecture docs, or task instructions may add stricter requirements. They may override these rules only when explicit and non-contradictory with safety, correctness, and user intent.

---

## 27. No Silent Dependency Introduction

Before adding a dependency:
- Check existing dependencies.
- Prefer existing project tools.
- Explain why the dependency is needed.
- Avoid large dependencies for small tasks.
- Avoid dependencies that conflict with the architecture.
- Avoid dependencies that create vendor lock-in.

---

## 28. No Untouchable Framework Magic

Do not introduce systems that make developers or agents reason case-by-case around hidden behavior. Prefer predictable, explicit, inspectable architecture.

---

## 29. Strong Typing Required

- No implicit `any` in TypeScript.
- No untyped business objects.
- No stringly-typed domain models when proper types are possible.
- Prefer generated types from schemas where available.
- In Rust: newtype IDs over bare strings; `#[repr(transparent)]`; `#[non_exhaustive]` on public enums.

---

## 30. Tests Are Part of Completion

Implementation is not complete until verified. Where available: run unit tests, integration tests, type checks, linters, build checks. Add tests for new behavior. Update tests when behavior intentionally changes.

---

## 31. Prefer Small, Reviewable Changes

Keep commits focused. Keep diffs small. Avoid broad rewrites. Avoid unrelated cleanup. Separate mechanical changes from behavioral changes.

---

## 32. Preserve Existing Behavior

Do not break existing behavior unless the task explicitly requires it. Identify current behavior, desired behavior, and compatibility impact before changing. Call out breaking changes clearly.

---

## 33. Security Is Not Optional

Always consider authentication, authorization, input validation, output escaping, secrets handling, tenant boundaries, data leakage, prompt injection, tool execution boundaries, and dependency risk. Never log secrets, tokens, credentials, private keys, or sensitive user data.

---

## 34. Agent Actions Must Be Auditable

For agentic systems, preserve an audit trail: user request, agent decision, tool calls, inputs, outputs, files changed, external effects, errors, human approvals where required.

---

## 35. Prefer Deterministic Systems

Prefer deterministic IDs, allocation algorithms, ordering, retries, replay, and conflict resolution. Non-determinism must be intentional and documented.

---

## 36. Local-First When Practical

Prefer architectures that can run locally and sync outward: local execution, local storage, offline-capable workflows, syncable state, portable runtimes, edge-compatible agents.

---

## 37. Runtime Portability Matters

Design for execution across cloud, local machine, mobile, browser, edge, WASM, and containerized environments. Avoid coupling business logic to a runtime unless required.

---

## 38. UI Is a Projection of State

The UI must not become the source of truth. UI renders state and submits intent. Backend/domain logic validates intent. Durable systems persist state. Events describe changes.

---

## 39. Artifacts Must Be Structured

Prometheus artifacts should be typed, versioned, inspectable, portable, renderable across supported hosts, compatible with agent workflows, and safe to persist and replay.

---

## 40. Stop When Done

When done: summarize what changed, summarize how it was verified, list remaining risks or follow-ups. Do not perform extra work unless asked.
