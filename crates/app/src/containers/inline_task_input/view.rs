use leptos::prelude::*;

#[component]
pub fn InlineTaskInputView(
    value: RwSignal<String>,
    input_ref: NodeRef<leptos::html::Input>,
    on_submit: Callback<String>,
    on_close: Callback<()>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("{class} pr-4 py-1")>
            <div class="flex items-center gap-2">
                <div class="flex-shrink-0">
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
                <input
                    type="text"
                    data-testid="task-detail-subtask-input"
                    class="flex-1 pt-0.5 bg-transparent border-none \
                           text-sm text-text-primary \
                           placeholder-text-tertiary \
                           focus:outline-none focus-visible:outline-none \
                           no-focus-ring"
                    placeholder="Task title..."
                    node_ref=input_ref
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        value.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            ev.stop_propagation();
                            let title =
                                value.get_untracked().trim().to_string();
                            if title.is_empty() {
                                return;
                            }
                            on_submit.run(title);
                        } else if ev.key() == "Escape" {
                            ev.stop_propagation();
                            on_close.run(());
                        }
                    }
                    on:blur=move |_| {
                        on_close.run(());
                    }
                />
            </div>
        </div>
    }
}
