//! The Resolve layer on top of the workspace: Intake → project context → ResolveOS guidance →
//! ResolvePM execution (the existing projects/tasks/pages/calendar) → validation (Hypothesis Zero).
//!
//! Everything here is plain data inside `WorkspaceData`, so it is saved, merged, backed up,
//! searched and exposed over MCP exactly like the rest of the workspace. No separate database.
//!
//! * `intake`   — the guided intake flow (data-driven questions and routes) and its sessions.
//! * `context`  — one `ProjectContext` per Resolve project: what intake learned, kept for every
//!                later step so nothing is asked twice.
//! * `records`  — decisions, hypotheses, experiments, evidence, risks, … linked to the project.
//! * `assess`   — deterministic project assessment: what's decided, uncertain, blocked, next.
//! * `roles`    — ResolveOS roles and runtime actions; prompts built from the context.

pub mod assess;
pub mod context;
pub mod intake;
pub mod journey;
pub mod records;
pub mod roles;

pub use context::ProjectContext;
pub use intake::Intake;
pub use records::{Kind, Record};
