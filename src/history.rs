//! Undo and soft delete for the whole workspace, with one mechanism.
//!
//! Any change worth undoing (delete, complete, adopt, an AI tool call) is wrapped in
//! `WorkspaceData::record_change`: the affected items are diffed before/after, and the previous
//! version of every changed or removed item is kept in `WorkspaceData.history`. Undo puts those
//! versions back (re-stamped so they win over the deletion when copies merge) and removes items the
//! change added. "Recently deleted" is the list of change sets that removed something.
//! History merges between devices like every other item; entries are kept 60 days (max 300).

use crate::model::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Change {
    /// Collection name: pages, projects, tasks, calendars, events, databases, records, intakes, contexts.
    pub coll: String,
    pub id: Id,
    /// The item before the change; None = the change added it.
    #[serde(default)]
    pub before: Option<Value>,
    /// The item's `updated` right after the change (0 if removed) — undo leaves later edits alone.
    #[serde(default)]
    pub after_updated: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ChangeSet {
    pub id: Id,
    pub label: String,
    /// "delete" | "complete" | "adopt" | "ai" | "edit"
    pub kind: String,
    pub changes: Vec<Change>,
    #[serde(default)]
    pub undone: bool,
    pub created: i64,
    pub updated: i64,
}

impl ChangeSet {
    pub fn removed(&self) -> usize {
        self.changes.iter().filter(|c| c.before.is_some() && c.after_updated == 0).count()
    }
    pub fn added(&self) -> usize {
        self.changes.iter().filter(|c| c.before.is_none()).count()
    }
}

type Snap = BTreeMap<(String, Id), (Value, i64)>;

fn put<T: Serialize>(snap: &mut Snap, coll: &str, items: &[T]) {
    for it in items {
        if let Ok(v) = serde_json::to_value(it) {
            let id = v["id"].as_str().unwrap_or("").to_string();
            let up = v["updated"].as_i64().unwrap_or(0);
            snap.insert((coll.to_string(), id), (v, up));
        }
    }
}

pub fn snapshot(d: &WorkspaceData) -> Snap {
    let mut s = Snap::new();
    put(&mut s, "pages", &d.pages);
    put(&mut s, "projects", &d.projects);
    put(&mut s, "tasks", &d.tasks);
    put(&mut s, "calendars", &d.calendars);
    put(&mut s, "events", &d.events);
    put(&mut s, "databases", &d.databases);
    put(&mut s, "records", &d.records);
    put(&mut s, "intakes", &d.intakes);
    put(&mut s, "contexts", &d.contexts);
    s
}

pub fn diff(before: &Snap, after: &Snap) -> Vec<Change> {
    let mut out = vec![];
    for (k, (v, _)) in before {
        match after.get(k) {
            None => out.push(Change { coll: k.0.clone(), id: k.1.clone(), before: Some(v.clone()), after_updated: 0 }),
            Some((v2, up2)) if v2 != v => out.push(Change { coll: k.0.clone(), id: k.1.clone(), before: Some(v.clone()), after_updated: *up2 }),
            _ => {}
        }
    }
    for (k, (_, up)) in after {
        if !before.contains_key(k) {
            out.push(Change { coll: k.0.clone(), id: k.1.clone(), before: None, after_updated: *up });
        }
    }
    out
}

fn updated_of<T: Serialize>(items: &[T], id: &str) -> Option<i64> {
    items.iter().filter_map(|x| serde_json::to_value(x).ok()).find(|v| v["id"] == id).and_then(|v| v["updated"].as_i64())
}

fn restore_into<T: serde::de::DeserializeOwned + Serialize>(items: &mut Vec<T>, id: &str, before: &Value, now: i64) -> bool {
    let mut v = before.clone();
    v["updated"] = Value::from(now);
    let Ok(item) = serde_json::from_value::<T>(v) else { return false };
    if let Some(pos) = items.iter().position(|x| serde_json::to_value(x).ok().map_or(false, |v| v["id"] == id)) {
        items[pos] = item;
    } else {
        items.push(item);
    }
    true
}

fn remove_from<T: Serialize>(items: &mut Vec<T>, id: &str) {
    items.retain(|x| serde_json::to_value(x).ok().map_or(true, |v| v["id"] != id));
}

impl WorkspaceData {
    /// Run `f`, and keep what it changed so it can be undone. Returns the change-set id
    /// (None if nothing changed).
    pub fn record_change(&mut self, label: &str, kind: &str, f: impl FnOnce(&mut WorkspaceData)) -> Option<Id> {
        let before = snapshot(self);
        f(self);
        self.keep_change(label, kind, &before)
    }

    /// Diff against an earlier snapshot and store the change set.
    pub fn keep_change(&mut self, label: &str, kind: &str, before: &Snap) -> Option<Id> {
        let after = snapshot(self);
        let changes = diff(before, &after);
        if changes.is_empty() {
            return None;
        }
        let t = now_ms();
        let cs = ChangeSet { id: new_id(), label: label.to_string(), kind: kind.to_string(), changes, undone: false, created: t, updated: t };
        let id = cs.id.clone();
        self.history.push(cs);
        // keep 60 days, at most 300 sets
        let cutoff = t - 60 * 24 * 3600 * 1000;
        self.history.retain(|c| c.created > cutoff);
        if self.history.len() > 300 {
            let n = self.history.len() - 300;
            self.history.drain(0..n);
        }
        Some(id)
    }

    /// Undo a change set. Returns (items restored or removed, items skipped because they were edited since).
    pub fn undo_change(&mut self, cs_id: &str) -> Result<(usize, usize), String> {
        let cs = self.history.iter().find(|c| c.id == cs_id).cloned().ok_or("That change is no longer in the history")?;
        if cs.undone {
            return Err("Already undone".into());
        }
        let now = now_ms();
        let (mut done, mut skipped) = (0, 0);
        for ch in cs.changes.iter().rev() {
            let current = match ch.coll.as_str() {
                "pages" => updated_of(&self.pages, &ch.id),
                "projects" => updated_of(&self.projects, &ch.id),
                "tasks" => updated_of(&self.tasks, &ch.id),
                "calendars" => updated_of(&self.calendars, &ch.id),
                "events" => updated_of(&self.events, &ch.id),
                "databases" => updated_of(&self.databases, &ch.id),
                "records" => updated_of(&self.records, &ch.id),
                "intakes" => updated_of(&self.intakes, &ch.id),
                "contexts" => updated_of(&self.contexts, &ch.id),
                _ => None,
            };
            // leave items that someone edited after this change
            if ch.after_updated != 0 && current.is_some() && current != Some(ch.after_updated) {
                skipped += 1;
                continue;
            }
            match &ch.before {
                None => {
                    match ch.coll.as_str() {
                        "pages" => remove_from(&mut self.pages, &ch.id),
                        "projects" => remove_from(&mut self.projects, &ch.id),
                        "tasks" => remove_from(&mut self.tasks, &ch.id),
                        "calendars" => remove_from(&mut self.calendars, &ch.id),
                        "events" => remove_from(&mut self.events, &ch.id),
                        "databases" => remove_from(&mut self.databases, &ch.id),
                        "records" => remove_from(&mut self.records, &ch.id),
                        "intakes" => remove_from(&mut self.intakes, &ch.id),
                        "contexts" => remove_from(&mut self.contexts, &ch.id),
                        _ => {}
                    }
                    self.bury(&ch.id);
                    done += 1;
                }
                Some(v) => {
                    let ok = match ch.coll.as_str() {
                        "pages" => restore_into(&mut self.pages, &ch.id, v, now),
                        "projects" => restore_into(&mut self.projects, &ch.id, v, now),
                        "tasks" => restore_into(&mut self.tasks, &ch.id, v, now),
                        "calendars" => restore_into(&mut self.calendars, &ch.id, v, now),
                        "events" => restore_into(&mut self.events, &ch.id, v, now),
                        "databases" => restore_into(&mut self.databases, &ch.id, v, now),
                        "records" => restore_into(&mut self.records, &ch.id, v, now),
                        "intakes" => restore_into(&mut self.intakes, &ch.id, v, now),
                        "contexts" => restore_into(&mut self.contexts, &ch.id, v, now),
                        _ => false,
                    };
                    if ok {
                        self.tombstones.retain(|t| t.id != ch.id);
                        done += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
        if let Some(c) = self.history.iter_mut().find(|c| c.id == cs_id) {
            c.undone = true;
            c.updated = now_ms();
        }
        Ok((done, skipped))
    }

    /// "Recently deleted": change sets that removed something and haven't been undone, newest first.
    pub fn recently_deleted(&self) -> Vec<&ChangeSet> {
        let mut v: Vec<&ChangeSet> = self.history.iter().filter(|c| !c.undone && c.removed() > 0).collect();
        v.sort_by_key(|c| std::cmp::Reverse(c.created));
        v
    }
}

/// A human label for a removed item (title / name).
pub fn item_label(c: &Change) -> String {
    let v = c.before.as_ref();
    let t = v.and_then(|v| v["title"].as_str().or(v["name"].as_str())).unwrap_or("item");
    let what = match c.coll.as_str() {
        "pages" => "page",
        "projects" => "project",
        "tasks" => "task",
        "calendars" => "calendar",
        "events" => "event",
        "databases" => "database",
        "records" => "record",
        "intakes" => "intake",
        "contexts" => "project context",
        _ => "item",
    };
    format!("{what}: {t}")
}
