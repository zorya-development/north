use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

#[component]
pub fn InlineTaskForm<F>(on_submit: F) -> impl IntoView
where
    F: Fn(String) + Send + Sync + 'static,
{
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let on_submit = std::sync::Arc::new(on_submit);

    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            if let Some(input) = input_ref.get() {
                let value = input.value().trim().to_string();
                if !value.is_empty() {
                    on_submit(value);
                    input.set_value("");
                }
            }
        }
    };

    view! {
        <div class="flex items-center gap-2 p-3 bg-white border border-peach-200 \
                    rounded-lg focus-within:border-teal-500 transition-colors">
            <span class="text-sage-400 text-sm">"+"</span>
            <input
                type="text"
                node_ref=input_ref
                placeholder="Add a task..."
                on:keydown=on_keydown
                class="flex-1 text-sm bg-transparent outline-none \
                       placeholder:text-sage-400 text-teal-950"
            />
        </div>
    }
}
