use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn InlineTaskInputView(
    value: RwSignal<String>,
    input_ref: NodeRef<leptos::html::Textarea>,
    on_submit: Callback<(String, Option<String>)>,
    on_close: Callback<()>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let auto_resize = move || {
        if let Some(el) = input_ref.get_untracked() {
            if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_el.style().set_property("height", "auto");
                let scroll_h = html_el.scroll_height();
                let _ = html_el
                    .style()
                    .set_property("height", &format!("{scroll_h}px"));
            }
        }
    };

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
                <textarea
                    data-testid="task-detail-subtask-input"
                    class="flex-1 pt-0.5 bg-transparent border-none \
                           text-sm text-text-primary \
                           placeholder-text-tertiary \
                           focus:outline-none focus-visible:outline-none \
                           no-focus-ring resize-none overflow-hidden"
                    placeholder="Task title..."
                    rows=1
                    node_ref=input_ref
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        value.set(event_target_value(&ev));
                        auto_resize();
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            if ev.ctrl_key() || ev.meta_key() {
                                // Ctrl+Enter: insert line break (default textarea behavior)
                                return;
                            }
                            // Plain Enter: submit
                            ev.prevent_default();
                            ev.stop_propagation();
                            let raw = value.get_untracked();
                            let mut lines = raw.splitn(2, '\n');
                            let title = lines.next().unwrap_or("").trim().to_string();
                            if title.is_empty() {
                                return;
                            }
                            let body = lines.next()
                                .map(|b| b.trim().to_string())
                                .filter(|b| !b.is_empty());
                            on_submit.run((title, body));
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
