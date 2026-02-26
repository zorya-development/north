use leptos::prelude::*;

/// Page-level context: call `keep(id)` when a task is completed so it stays
/// visible in the list until the next page refresh / navigation.
/// Provided by page controllers, consumed by TaskCheckbox and TTL controllers.
#[derive(Clone, Copy, Default)]
pub struct KeepCompletedVisible(RwSignal<Vec<i64>>);

impl KeepCompletedVisible {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keep(&self, id: i64) {
        self.0.update(|ids| {
            if !ids.contains(&id) {
                ids.push(id);
            }
        });
    }

    pub fn contains(&self, id: i64) -> bool {
        self.0.get_untracked().contains(&id)
    }

    pub fn signal(&self) -> RwSignal<Vec<i64>> {
        self.0
    }
}
