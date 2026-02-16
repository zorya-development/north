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
        <div class=format!("{class} pl-6 py-1")>
            <input
                type="text"
                class="w-full bg-transparent border-none \
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
    }
}
