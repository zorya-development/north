use std::collections::HashMap;

use leptos::prelude::*;

/// Reactive proxy over localStorage.
/// Uses a single `RwSignal<HashMap>` so all reads subscribe to the same
/// signal — any key change notifies all readers. With only ~5 boolean
/// flags this is negligible, and it avoids stale per-key signals when
/// created inside reactive scopes.
#[derive(Clone, Copy)]
pub struct BrowserStorageStore {
    map: RwSignal<HashMap<String, bool>>,
}

impl Default for BrowserStorageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserStorageStore {
    pub fn new() -> Self {
        Self {
            map: RwSignal::new(HashMap::new()),
        }
    }

    /// Get a boolean value reactively.
    /// On first access, reads from localStorage and seeds the map.
    pub fn get_bool(&self, key: &str) -> bool {
        self.ensure_loaded(key);
        self.map.with(|m| m.get(key).copied().unwrap_or(false))
    }

    /// Set a boolean value — updates the reactive map and localStorage.
    pub fn set_bool(&self, key: &str, value: bool) {
        self.map.update(|m| {
            m.insert(key.to_string(), value);
        });
        Self::write_to_storage(key, value);
    }

    /// Toggle a boolean value. Returns the new value.
    pub fn toggle_bool(&self, key: &str) -> bool {
        self.ensure_loaded(key);
        let current = self
            .map
            .with_untracked(|m| m.get(key).copied().unwrap_or(false));
        let new_val = !current;
        self.set_bool(key, new_val);
        new_val
    }

    /// Seed the key from localStorage if not already present (untracked).
    fn ensure_loaded(&self, key: &str) {
        let exists = self.map.with_untracked(|m| m.contains_key(key));
        if !exists {
            let val = Self::read_from_storage(key);
            self.map.update_untracked(|m| {
                m.insert(key.to_string(), val);
            });
        }
    }

    fn read_from_storage(key: &str) -> bool {
        #[cfg(feature = "hydrate")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok()?)
                .and_then(|s| s.get_item(key).ok()?)
                .map(|v| v == "true")
                .unwrap_or(false)
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = key;
            false
        }
    }

    fn write_to_storage(key: &str, value: bool) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()?) {
                let _ = storage.set_item(key, if value { "true" } else { "false" });
            }
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = (key, value);
        }
    }
}
