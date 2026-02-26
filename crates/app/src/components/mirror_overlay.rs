use leptos::prelude::*;

/// Transparent overlay that renders the first line of text in primary color
/// and subsequent lines in a muted color. Layered on top of a textarea
/// whose text is made transparent via the `textarea-mirror` utility class.
#[component]
pub fn MirrorOverlay(value: Signal<String>) -> impl IntoView {
    let lines = move || {
        let raw = value.try_get().unwrap_or_default();
        if raw.is_empty() {
            return (String::new(), None);
        }
        match raw.split_once('\n') {
            Some((first, rest)) => (first.to_string(), Some(format!("\n{rest}"))),
            None => (raw, None),
        }
    };

    view! {
        <div
            class="absolute inset-0 pt-0.5 pointer-events-none text-sm whitespace-pre-wrap break-words overflow-hidden"
            aria-hidden="true"
        >
            <span class="text-text-primary">{move || lines().0}</span>
            {move || {
                lines().1.map(|rest| {
                    view! { <span class="text-text-secondary">{rest}</span> }
                })
            }}
        </div>
    }
}
