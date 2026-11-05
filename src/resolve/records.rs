//! Project knowledge records: the Resolve layer's structured memory for a project.
//!
//! One storage type (`Record`) with a `kind` and a small, data-driven field schema per kind,
//! instead of a struct + table + form per entity. Every record carries `updated`, so it merges
//! between devices exactly like pages and tasks do, and links to tasks, pages and other records.
//!
//! Hypothesis Zero (H0) mapping — see docs/RESOLVE_SPEC.md §Validation:
//! * `Hypothesis` = an Hx node. `parent_id` makes the tree recursive (no fixed depth or order).
//!   The explicit negation NHx and alternative explanations are fields of the node.
//!   `state` is the *evidential* state: Unresolved / Supported / Weakened / Refuted.
//! * `Experiment` = an E(Hx) branch; its parent is the Hx it tests (any node, not just leaves).
//!   Fields follow the H0 page's "minimum usable specification". Once an experiment is
//!   started, its thresholds are locked ("do not retroactively tune thresholds").
//! * `Evidence` records whether something is a fact, assumption, inference or evidence,
//!   its quality, and whether it supports or contradicts; contradicting evidence is kept.
//! * `Decision` links to its provenance (records) and has revisit conditions.

use crate::model::{new_id, now_ms, Id};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord)]
pub enum Kind {
    #[default]
    Hypothesis,
    Experiment,
    Evidence,
    Decision,
    Risk,
    Assumption,
    Requirement,
    Dependency,
    Question,
    Blocker,
    Milestone,
    Update,
    /// Output of a ResolveOS role / guidance step, kept so context survives sessions.
    Guidance,
}

pub struct FieldDef {
    pub key: &'static str,
    pub label: &'static str,
    pub multiline: bool,
    /// Locked once an experiment is running (H0: thresholds are predefined).
    pub lock_when_running: bool,
}

const fn f(key: &'static str, label: &'static str, multiline: bool) -> FieldDef {
    FieldDef { key, label, multiline, lock_when_running: false }
}
const fn locked(key: &'static str, label: &'static str) -> FieldDef {
    FieldDef { key, label, multiline: true, lock_when_running: true }
}

impl Kind {
    pub const ALL: [Kind; 13] = [
        Kind::Hypothesis,
        Kind::Experiment,
        Kind::Evidence,
        Kind::Decision,
        Kind::Risk,
        Kind::Assumption,
        Kind::Requirement,
        Kind::Dependency,
        Kind::Question,
        Kind::Blocker,
        Kind::Milestone,
        Kind::Update,
        Kind::Guidance,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Kind::Hypothesis => "Hypothesis",
            Kind::Experiment => "Experiment",
            Kind::Evidence => "Evidence",
            Kind::Decision => "Decision",
            Kind::Risk => "Risk",
            Kind::Assumption => "Assumption",
            Kind::Requirement => "Requirement",
            Kind::Dependency => "Dependency",
            Kind::Question => "Open question",
            Kind::Blocker => "Blocker",
            Kind::Milestone => "Milestone",
            Kind::Update => "Progress update",
            Kind::Guidance => "Guidance",
        }
    }
    pub fn plural(self) -> &'static str {
        match self {
            Kind::Hypothesis => "Hypotheses",
            Kind::Experiment => "Experiments",
            Kind::Evidence => "Evidence",
            Kind::Decision => "Decisions",
            Kind::Risk => "Risks",
            Kind::Assumption => "Assumptions",
            Kind::Requirement => "Requirements",
            Kind::Dependency => "Dependencies",
            Kind::Question => "Open questions",
            Kind::Blocker => "Blockers",
            Kind::Milestone => "Milestones",
            Kind::Update => "Updates",
            Kind::Guidance => "Guidance",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Hypothesis => "H",
            Kind::Experiment => "E",
            Kind::Evidence => "◆",
            Kind::Decision => "✔",
            Kind::Risk => "⚠",
            Kind::Assumption => "~",
            Kind::Requirement => "≡",
            Kind::Dependency => "⛓",
            Kind::Question => "?",
            Kind::Blocker => "⛔",
            Kind::Milestone => "⚑",
            Kind::Update => "↻",
            Kind::Guidance => "✦",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Kind::Hypothesis => "hypothesis",
            Kind::Experiment => "experiment",
            Kind::Evidence => "evidence",
            Kind::Decision => "decision",
            Kind::Risk => "risk",
            Kind::Assumption => "assumption",
            Kind::Requirement => "requirement",
            Kind::Dependency => "dependency",
            Kind::Question => "question",
            Kind::Blocker => "blocker",
            Kind::Milestone => "milestone",
            Kind::Update => "update",
            Kind::Guidance => "guidance",
        }
    }
    pub fn parse(s: &str) -> Option<Kind> {
        let s = s.trim().to_lowercase();
        Kind::ALL.iter().copied().find(|k| k.key() == s || k.label().to_lowercase() == s || k.plural().to_lowercase() == s)
    }
    /// Allowed values of `status` for this kind (first = default).
    pub fn statuses(self) -> &'static [&'static str] {
        match self {
            Kind::Hypothesis => &["Unresolved", "Supported", "Weakened", "Refuted"],
            Kind::Experiment => &["Planned", "Running", "Done", "Stopped"],
            Kind::Evidence => &["Recorded", "Superseded"],
            Kind::Decision => &["Proposed", "Decided", "Revisit", "Superseded"],
            Kind::Risk => &["Open", "Mitigating", "Closed", "Occurred"],
            Kind::Assumption => &["Untested", "Holding", "Broken"],
            Kind::Requirement => &["Draft", "Agreed", "Done", "Dropped"],
            Kind::Dependency => &["Waiting", "Met", "At risk"],
            Kind::Question => &["Open", "Answered"],
            Kind::Blocker => &["Blocking", "Resolved"],
            Kind::Milestone => &["Planned", "Reached", "Missed"],
            Kind::Update | Kind::Guidance => &["Recorded"],
        }
    }
    /// Is a record in this status still something to act on?
    pub fn is_open(self, status: &str) -> bool {
        match self {
            Kind::Hypothesis => status == "Unresolved" || status == "Weakened",
            Kind::Experiment => status == "Planned" || status == "Running",
            Kind::Decision => status == "Proposed" || status == "Revisit",
            Kind::Risk => status == "Open" || status == "Mitigating",
            Kind::Assumption => status == "Untested",
            Kind::Requirement => status == "Draft" || status == "Agreed",
            Kind::Dependency => status == "Waiting" || status == "At risk",
            Kind::Question => status == "Open",
            Kind::Blocker => status == "Blocking",
            Kind::Milestone => status == "Planned",
            Kind::Evidence | Kind::Update | Kind::Guidance => false,
        }
    }
    pub fn fields(self) -> &'static [FieldDef] {
        const HYP: &[FieldDef] = &[
            f("negation", "Explicit negation (NHx)", true),
            f("alternatives", "Alternative / competing explanations", true),
            f("type", "Type (causal, intervention, assumption about evidence…)", false),
            f("confidence", "Confidence (your judgement, e.g. 60%)", false),
            f("uncertainty", "Material uncertainty", true),
            f("decision", "Decision at stake", true),
            f("revisit", "Revisit when…", true),
        ];
        const EXP: &[FieldDef] = &[
            f("decision", "Decision at stake, why this test matters, action if it changes", true),
            locked("prediction", "Observable prediction that differs between Hx and NHx"),
            f("test_type", "Test type (research, interview, prototype, A/B, pilot…)", false),
            f("method", "Method, unit of analysis, sample and selection", true),
            f("baseline", "Baseline / comparison (or why none), source quality, biases, confounders", true),
            locked("support_if", "Supports Hx if…"),
            locked("refute_if", "Refutes / weakens Hx if…"),
            locked("inconclusive_if", "Inconclusive if…"),
            f("limits", "Feasibility limits, stop / revisit conditions", true),
            f("owner", "Owner", false),
            f("date", "Execution date", false),
            f("sources", "Data / source links", true),
            f("observations", "Raw observations", true),
            f("limitations", "Observed limitations", true),
            f("result", "Result (supports / weakens / refutes / inconclusive)", false),
            f("update", "Update to Hx, NHx, confidence, dependent nodes, decision", true),
            f("next", "Next experiment", true),
        ];
        const EVI: &[FieldDef] = &[
            f("nature", "Nature: fact, assumption, inference or evidence", false),
            f("direction", "Supports, contradicts or neutral", false),
            f("quality", "Source quality", false),
            f("source", "Source (link, document, person)", true),
            f("date", "Date observed", false),
        ];
        const DEC: &[FieldDef] = &[
            f("options", "Options considered", true),
            f("rationale", "Rationale", true),
            f("uncertainty", "Remaining uncertainty", true),
            f("revisit", "Revisit conditions", true),
            f("date", "Date", false),
            f("owner", "Decided by", false),
        ];
        const RISK: &[FieldDef] = &[f("impact", "Impact", false), f("likelihood", "Likelihood", false), f("mitigation", "Mitigation", true), f("owner", "Owner", false)];
        const ASM: &[FieldDef] = &[f("why", "Why we believe it", true), f("test", "How it could be tested", true)];
        const REQ: &[FieldDef] = &[f("acceptance", "Acceptance criteria", true), f("source", "Source / who asked", false), f("priority", "Priority", false)];
        const DEP: &[FieldDef] = &[f("on", "Depends on (who / what)", false), f("needed_by", "Needed by", false)];
        const Q: &[FieldDef] = &[f("answer", "Answer", true)];
        const BLK: &[FieldDef] = &[f("unblock", "What would unblock it", true), f("owner", "Owner", false)];
        const MIL: &[FieldDef] = &[f("date", "Target date", false), f("criteria", "Done when…", true)];
        const UPD: &[FieldDef] = &[f("date", "Date", false)];
        const GUI: &[FieldDef] = &[f("role", "Role", false), f("action", "Action", false), f("provider", "Model / provider", false)];
        match self {
            Kind::Hypothesis => HYP,
            Kind::Experiment => EXP,
            Kind::Evidence => EVI,
            Kind::Decision => DEC,
            Kind::Risk => RISK,
            Kind::Assumption => ASM,
            Kind::Requirement => REQ,
            Kind::Dependency => DEP,
            Kind::Question => Q,
            Kind::Blocker => BLK,
            Kind::Milestone => MIL,
            Kind::Update => UPD,
            Kind::Guidance => GUI,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Record {
    pub id: Id,
    pub kind: Kind,
    /// The Resolve project this belongs to (a workspace project id).
    #[serde(default)]
    pub project_id: Option<Id>,
    /// Hypothesis tree (Hx → sub-hypotheses, Hx → experiments) or any other nesting.
    #[serde(default)]
    pub parent_id: Option<Id>,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
    /// Linked items: task ids, page ids, other record ids (provenance, dependencies).
    #[serde(default)]
    pub links: Vec<Id>,
    pub created: i64,
    pub updated: i64,
}

impl Record {
    pub fn new(kind: Kind, project_id: Option<Id>, title: impl Into<String>) -> Self {
        let t = now_ms();
        Record {
            id: new_id(),
            kind,
            project_id,
            parent_id: None,
            title: title.into(),
            body: String::new(),
            status: kind.statuses()[0].to_string(),
            fields: BTreeMap::new(),
            links: vec![],
            created: t,
            updated: t,
        }
    }
    pub fn field(&self, k: &str) -> &str {
        self.fields.get(k).map(|s| s.as_str()).unwrap_or("")
    }
    pub fn is_open(&self) -> bool {
        self.kind.is_open(&self.status)
    }
    /// H0 rule: an experiment's predictions and thresholds are frozen once it has started.
    pub fn is_locked(&self, key: &str) -> bool {
        self.kind == Kind::Experiment
            && self.status != "Planned"
            && self.kind.fields().iter().any(|f| f.key == key && f.lock_when_running)
    }
    /// Set a status if it is valid for the kind.
    pub fn set_status(&mut self, s: &str) -> Result<(), String> {
        match self.kind.statuses().iter().find(|x| x.eq_ignore_ascii_case(s.trim())) {
            Some(v) => {
                self.status = v.to_string();
                self.updated = now_ms();
                Ok(())
            }
            None => Err(format!("'{s}' isn't a status for {}: use one of {}", self.kind.label(), self.kind.statuses().join(", "))),
        }
    }
    /// Set a field, refusing locked ones. Unknown keys are kept (forward compatible).
    pub fn set_field(&mut self, k: &str, v: &str) -> Result<(), String> {
        if self.is_locked(k) && self.field(k) != v {
            return Err(format!("'{k}' was fixed when the experiment started; record a new experiment instead of changing its thresholds."));
        }
        if v.is_empty() {
            self.fields.remove(k);
        } else {
            self.fields.insert(k.to_string(), v.to_string());
        }
        self.updated = now_ms();
        Ok(())
    }
}
