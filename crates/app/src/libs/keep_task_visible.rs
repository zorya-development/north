use leptos::prelude::*;

/// Page-level context: call `keep(id)` to pin a task in the current list
/// even after it stops matching the page's base filter.
/// Provided by page controllers, consumed by TTL and anywhere else needed.
#[derive(Clone, Copy)]
pub struct KeepTaskVisible(RwSignal<Vec<i64>>);

impl KeepTaskVisible {
    pub fn new(signal: RwSignal<Vec<i64>>) -> Self {
        Self(signal)
    }

    pub fn keep(&self, id: i64) {
        self.0.update(|ids| {
            if !ids.contains(&id) {
                ids.push(id);
            }
        });
    }
}
