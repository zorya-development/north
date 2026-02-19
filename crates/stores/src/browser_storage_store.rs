use std::collections::HashMap;

use leptos::prelude::*;

/// Reactive proxy over localStorage.
/// Each key gets a lazily-created `RwSignal<bool>` initialized from
/// localStorage. Writes update both the signal and localStorage.
#[derive(Clone, Copy)]
pub struct BrowserStorageStore {
    signals: RwSignal<HashMap<String, RwSignal<bool>>>,
}

impl Default for BrowserStorageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserStorageStore {
    pub fn new() -> Self {
        Self {
            signals: RwSignal::new(HashMap::new()),
        }
    }

    /// Get a boolean value reactively.
    /// On first access, reads from localStorage and caches the signal.
    pub fn get_bool(&self, key: &str) -> bool {
        self.signal_for(key).get()
    }

    /// Set a boolean value — updates the reactive signal and localStorage.
    pub fn set_bool(&self, key: &str, value: bool) {
        self.signal_for(key).set(value);
        Self::write_to_storage(key, value);
    }

    /// Toggle a boolean value. Returns the new value.
    pub fn toggle_bool(&self, key: &str) -> bool {
        let sig = self.signal_for(key);
        let new_val = !sig.get_untracked();
        sig.set(new_val);
        Self::write_to_storage(key, new_val);
        new_val
    }

    fn signal_for(&self, key: &str) -> RwSignal<bool> {
        if let Some(sig) = self.signals.with_untracked(|m| m.get(key).copied()) {
            return sig;
        }
        let val = Self::read_from_storage(key);
        let sig = RwSignal::new(val);
        self.signals.update_untracked(|m| {
            m.insert(key.to_string(), sig);
        });
        sig
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
