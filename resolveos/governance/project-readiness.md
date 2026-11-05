---
type: governance
scope: global
owner: ResolveOS
version: 0.1
status: draft
source_project: ResolveYGO adoption validation
source_files:
  - migration/architecture-review-3.md
  - migration/resolvepm-residual-audit.md
  - migration/template-layer-review.md
related_context:
  - 01-context/running-context.md
  - 01-context/project-loading-rules.md
  - 01-context/context-loading-rules.md
  - 01-context/missing-context-behaviour.md
related_workflows:
  - 05-workflows/project-initiation.md
related_governance:
  - 06-governance/source-of-truth-rules.md
  - 06-governance/update-process.md
  - 06-governance/architecture-decisions.md
review_required: true
---

# Purpose

Define readiness assessment across project lifecycles.

Project readiness is not a single state. A project may be ready for adoption, discovery, planning, implementation, or validation at different times.

Being ready for one stage does not imply readiness for another.

This governance exists because the ResolveYGO adoption validation showed that a project can be suitable for ResolveOS adoption while still needing separate checks for canonical source, current state, tracker/repository consistency, implementation readiness, and validation evidence.

Readiness language should help the user move forward. Do not surface a readiness gap without also identifying whether it blocks current progress, the smallest mitigation, and the next action.

Source references:

- `migration/architecture-review-3.md` > Project Initiation Assessment
- `05-workflows/project-initiation.md` > Project Setup Report and continuation state
- `01-context/project-loading-rules.md` > project source ownership
- `06-governance/source-of-truth-rules.md` > source ownership and conflict rules

# Rules

Assess readiness independently for each stage:

- adoption readiness
- discovery readiness
- planning readiness
- implementation readiness
- validation readiness

Do not treat readiness as transitive.

Do not say a project is implementation ready because it is adoption ready.

Do not say a project is validation ready because implementation exists.

Do not hide uncertainty about readiness. If evidence is missing, mark the readiness state as blocked, partial, or unknown.

Do not describe a project as not ready without explaining the practical resolution path.

Readiness assessment should identify:

- purpose
- required evidence
- common blockers
- minimum conditions
- anti-patterns
- whether a gap blocks progress now
- what can still continue safely, if anything
- the smallest mitigation or source needed
- which user-facing role, team member, specialist chat, AI assistant, engineer, consultant, or source owner should handle it
- whether a prompt, handoff, checklist, or follow-up task should be generated

Keep readiness output proportionate. Initial setup should not list every readiness category unless it helps the user understand the project state or next action.

# Validation State Model

Use a lightweight validation state model when assumptions, evidence, confidence, or validation status affect readiness.

Core concepts:

- Assumption: a claim, dependency, expectation, user need, business rule, technical belief, or delivery condition that is not yet fully proven.
- Evidence: a source-backed observation, test result, research finding, customer signal, repository state, completion report, validation report, or authoritative source-system record that supports or weakens an assumption.
- Confidence: the current strength of belief based on available evidence, stated as high, medium, low, or unknown.
- Validation status: proven, partially validated, unvalidated, or disproven.

Validation statuses mean:

| Status | Meaning |
| --- | --- |
| Proven | Strong evidence supports the claim for the current project context. |
| Partially validated | Some evidence supports the claim, but scope, sample size, environment, or acceptance coverage is incomplete. |
| Unvalidated | The claim may be plausible, but source-backed evidence is missing or too weak to rely on. |
| Disproven | Evidence contradicts the claim or shows the expected result is false in the current context. |

Use this model to improve readiness assessment, commercial validation, product validation, implementation validation, and project continuation.

Do not turn validation state into a research-management system. Keep exact research plans, customer interviews, analytics, test commands, tracker fields, and acceptance records project-owned.

# Adoption Readiness

## Purpose

Determine whether ResolveOS can be safely applied to a project without overwriting existing project state or inventing missing project facts.

## Required Evidence

- project identity
- project repository, workspace, or primary working location
- known source systems
- project owner or admin instruction
- existing project context, if any
- existing operating model, if any
- clear statement of whether this is new project initiation, existing project adoption, or continuation

## Common Blockers

- no accessible project source
- unclear project identity
- multiple candidate repositories or implementations with no canonical source
- no admin approval to assess or adopt
- project-specific doctrine being mistaken for ResolveOS doctrine

## Minimum Conditions

- The project can be identified.
- The available sources can be named.
- Existing state will be preserved.
- ResolveOS can make recommendations without creating files or changing systems automatically.

## Anti-Patterns

- Assume every project needs every ResolveOS role, chat, workflow, or template.
- Start by creating bootstrap files.
- Overwrite existing project structure.
- Import product doctrine into ResolveOS.
- Treat adoption readiness as implementation readiness.

# Discovery Readiness

## Purpose

Determine whether enough context exists to investigate, classify, and understand the project safely.

## Required Evidence

- canonical project source or list of candidate sources
- documentation sources
- tracker or task source, if one exists
- repository or implementation source, if one exists
- known gaps, contradictions, and unavailable sources
- project-specific context-loading constraints

## Common Blockers

- unclear canonical source
- inaccessible repository, tracker, database, or documentation
- contradictory project records
- stale metadata
- missing owner for project decisions

## Minimum Conditions

- The investigation can identify what is known, unknown, missing, stale, or contradictory.
- The assistant can distinguish source facts from assumptions.
- Discovery can proceed without changing project state.

## Anti-Patterns

- Treat stale documentation as current implementation state.
- Treat tracker status as repository truth without checking.
- Treat repository state as product approval.
- Smooth over contradictions instead of surfacing them.

# Planning Readiness

## Purpose

Determine whether the project has enough source-backed context to recommend roles, chats, context, source-of-truth structure, gaps, highest-leverage activity, and top recommended actions.

## Required Evidence

- project objective
- canonical source assessment
- known source systems
- current or proposed operating model
- known constraints
- active work or intended next work
- known risks and blockers
- relevant decisions or decision gaps
- validation state for assumptions that materially affect the next recommendation

## Common Blockers

- objective is unclear
- source ownership is unresolved
- active work is unknown
- plan conflicts with repository reality
- role or chat recommendations would be speculative

## Minimum Conditions

- The highest-leverage activity can be stated with evidence.
- Missing context is identified rather than guessed.
- Project-specific facts remain project-owned.
- Recommendations can be reviewed by the admin or source owner.

## Anti-Patterns

- Create a full operating model without source evidence.
- Recommend every role or chat by default.
- Treat planning as approval to implement.
- Hide project-specific assumptions inside global ResolveOS guidance.

# Implementation Readiness

## Purpose

Determine whether scoped implementation work can begin safely.

## Required Evidence

- approved ticket, task, batch, or scope
- current objective
- acceptance criteria or explicit acceptance gap
- dependency and blocker state
- canonical repository and implementation location
- relevant project context
- latest completion, blocker, or decision evidence where continuation is involved
- applicable validation expectations

## Common Blockers

- missing approved scope
- missing or guessed acceptance criteria
- unresolved canonical source
- tracker and repository state disagree
- required dependency is incomplete
- latest completion evidence is missing
- active blocker is unresolved

## Minimum Conditions

- Current scope is clear.
- Required context is available or the limitation is explicit.
- Dependencies and blockers have been checked.
- Continuing work is supported by evidence rather than memory.

## Anti-Patterns

- Start implementation from chat memory.
- Treat planning readiness as implementation readiness.
- Implement future-ticket work.
- Work around blockers silently.
- Claim implementation can begin when canonical source or active scope is unresolved.

# Validation Readiness

## Purpose

Determine whether delivered work can be checked, reviewed, or accepted with enough evidence.

## Required Evidence

- implementation scope
- acceptance criteria
- validation method
- project-specific commands or manual steps where available
- repository state or delivered artifact
- completion report, if work was already performed
- known failed, skipped, blocked, or not-run checks
- assumptions being validated
- evidence and confidence for the validation result

## Common Blockers

- no acceptance criteria
- no validation path
- implementation location is unclear
- tracker says complete but repository evidence is missing
- failed checks are unresolved
- validation environment is unavailable

## Minimum Conditions

- There is something specific to validate.
- The expected result is observable.
- Validation evidence can be captured or missing validation can be reported honestly.
- Failed or unavailable checks are not hidden.

## Anti-Patterns

- Treat implementation existence as validation success.
- Treat tracker completion as validation evidence.
- Claim checks passed when they did not run.
- Hide failed validation inside a positive completion summary.

# Enforcement

Project initiation, continuation, implementation planning, and validation work should state the relevant readiness state when readiness affects the highest-leverage activity.

If readiness is partial, blocked, or unknown, report the blocker or uncertainty clearly.

Also report the practical resolution path. If the issue does not block the current objective, say so and continue with an explicit limitation.

Where readiness depends on project-specific systems, keep exact commands, tools, fields, ticket keys, repositories, databases, and implementation details in the project repository or source system.

# Exceptions

Small low-risk conversations may not need a full readiness assessment.

Do not over-format casual questions into readiness reports.

When work is significant, durable, implementation-facing, validation-facing, or project-adoption-facing, readiness should be assessed explicitly.

# Examples

Adoption ready but not implementation ready:

```text
The project has an accessible repository and source documentation, so ResolveOS can assess adoption.
Implementation should not start yet because the active ticket, acceptance criteria, and latest completion evidence are missing.
Smallest mitigation: provide the active ticket or latest handoff, or approve a clearly labelled implementation-readiness draft.
```

Planning ready but not validation ready:

```text
The next work can be planned from the source context and tracker.
Validation is not ready because the project-specific check commands and expected manual result are not documented.
Smallest mitigation: document the validation path before claiming acceptance or release readiness.
```

# Notes

Deferred because they belong elsewhere:

- Project-specific readiness checklists belong in project repositories.
- Exact tracker fields, repository names, database names, commands, and environment details belong in project context.
- Source-system posting rules belong in future source-system handoff guidance or project-owned process.

Ambiguous for admin review:

- Whether project readiness should later have a template.
- Whether adoption validation should become a repeatable workflow after more project trials.
- Whether customer-facing readiness language should differ from internal agent-facing readiness language.
