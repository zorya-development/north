use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct StatusBarMessage {
    pub text: String,
    pub variant: StatusBarVariant,
    pub style: StatusBarStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusBarVariant {
    Danger,
    Info,
    Success,
}

/// Controls how the status bar message is displayed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusBarStyle {
    /// Persistent message with animated spinner (e.g. loading states).
    Spinner,
    /// Auto-dismissing toast notification (disappears after duration).
    Toast,
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
            style: StatusBarStyle::Spinner,
        }));
    }

    /// Show a toast notification that auto-dismisses after a duration.
    /// The actual timeout is handled by the StatusBar component.
    pub fn notify(&self, variant: StatusBarVariant, text: impl Into<String>) {
        self.message.set(Some(StatusBarMessage {
            text: text.into(),
            variant,
            style: StatusBarStyle::Toast,
        }));
    }

    pub fn hide_message(&self) {
        self.message.set(None);
    }

    pub fn is_visible(&self) -> bool {
        self.message.get_untracked().is_some()
    }
}
