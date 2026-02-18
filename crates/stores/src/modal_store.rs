use std::collections::HashMap;

use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ModalStore {
    modals: RwSignal<HashMap<String, bool>>,
}

impl Default for ModalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ModalStore {
    pub fn new() -> Self {
        Self {
            modals: RwSignal::new(HashMap::new()),
        }
    }

    pub fn open(&self, name: &str) {
        self.modals.update(|m| {
            m.insert(name.to_string(), true);
        });
    }

    pub fn close(&self, name: &str) {
        self.modals.update(|m| {
            m.remove(name);
        });
    }

    pub fn is_open(&self, name: &str) -> bool {
        self.modals.get().get(name).copied().unwrap_or(false)
    }

    pub fn is_any_open(&self) -> bool {
        !self.modals.get().is_empty()
    }
}
