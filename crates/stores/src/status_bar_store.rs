use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct StatusBarMessage {
    pub text: String,
    pub variant: StatusBarVariant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusBarVariant {
    Danger,
    Info,
}

#[derive(Clone, Copy)]
pub struct StatusBarStore {
    pub message: RwSignal<Option<StatusBarMessage>>,
}

impl Default for StatusBarStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBarStore {
    pub fn new() -> Self {
        Self {
            message: RwSignal::new(None),
        }
    }

    pub fn show_message(&self, text: impl Into<String>, variant: StatusBarVariant) {
        self.message.set(Some(StatusBarMessage {
            text: text.into(),
            variant,
        }));
    }

    pub fn hide_message(&self) {
        self.message.set(None);
    }

    pub fn is_visible(&self) -> bool {
        self.message.get_untracked().is_some()
    }
}
