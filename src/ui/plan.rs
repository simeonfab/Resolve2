//! Plan my day (B-E21, B-E22, B-E23): a sheet over Home. The AI is asked for a structured plan
//! (time, item, reason, what won't fit); if it fails, Home says so in its own card and lists
//! today by time.

use super::*;

/// The Plan my day sheet's state.
#[derive(Default)]
pub struct PlanUi {
    /// The sheet is open.
    pub open: bool,
    /// The last attempt failed: what went wrong (Home shows it in its own card, B-E6).
    pub error: Option<String>,
}

impl App {
    /// Open Plan my day.
    pub fn open_plan_day(&mut self) {
        self.plan.open = true;
    }
}
