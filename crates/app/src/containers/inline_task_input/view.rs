use leptos::prelude::*;

use crate::containers::smart_textarea::SmartTextarea;

#[component]
pub fn InlineTaskInputView(
    value: RwSignal<String>,
    input_ref: NodeRef<leptos::html::Textarea>,
    on_submit: Callback<(String, Option<String>)>,
    on_close: Callback<()>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let submit_cb = Callback::new(move |()| {
        let raw = value.get_untracked();
        let mut lines = raw.splitn(2, '\n');
        let title = lines.next().unwrap_or("").trim().to_string();
        if title.is_empty() {
            return;
        }
        let body = lines
            .next()
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty());
        on_submit.run((title, body));
    });

    view! {
        <div class=format!("{class} pr-4 py-1")>
            <div class="flex items-start gap-2">
                <div class="flex-shrink-0 pt-1">
                    <svg width="16" height="16" viewBox="0 0 16 16">
                        <circle
                            cx="8" cy="8" r="6.5"
                            fill="none"
                            stroke="var(--text-secondary)"
                            stroke-width="2"
                            opacity="0.5"
                        />
                    </svg>
                </div>
                <SmartTextarea
                    value=value
                    placeholder="Task title..."
                    data_testid="task-detail-subtask-input"
                    autocomplete=true
                    mirror_overlay=true
                    auto_resize=true
                    multiline=true
                    autofocus=true
                    node_ref=input_ref
                    on_submit=submit_cb
                    on_close=on_close
                    on_blur=on_close
                    class="flex-1 w-full pt-0.5 bg-transparent border-none \
                           text-sm \
                           focus:outline-none focus-visible:outline-none \
                           no-focus-ring resize-none overflow-hidden"
                />
            </div>
        </div>
    }
}
