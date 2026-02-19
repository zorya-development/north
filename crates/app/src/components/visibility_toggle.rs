use leptos::prelude::*;

#[cfg(feature = "hydrate")]
mod js {
    use leptos::wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = "
        export function get_visibility(key) {
            return localStorage.getItem(key) === 'true';
        }
        export function set_visibility(key, val) {
            localStorage.setItem(key, val ? 'true' : 'false');
        }
    ")]
    extern "C" {
        pub fn get_visibility(key: &str) -> bool;
        pub fn set_visibility(key: &str, val: bool);
    }
}

#[component]
pub fn VisibilityToggle(page_key: String, hide_non_actionable: RwSignal<bool>) -> impl IntoView {
    let storage_key = format!("north-hide-non-actionable-{page_key}");

    #[cfg(feature = "hydrate")]
    {
        let key = storage_key.clone();
        Effect::new(move |_| {
            let val = js::get_visibility(&key);
            hide_non_actionable.set(val);
        });
    }

    let toggle = {
        #[allow(unused_variables)]
        let key = storage_key;
        move |_| {
            let new_val = !hide_non_actionable.get_untracked();
            hide_non_actionable.set(new_val);

            #[cfg(feature = "hydrate")]
            js::set_visibility(&key, new_val);
        }
    };

    view! {
        <button
            on:click=toggle
            class="text-xs text-text-secondary \
                   hover:text-text-primary transition-colors"
        >
            {move || {
                if hide_non_actionable.get() {
                    "Show all tasks"
                } else {
                    "Hide non-actionable"
                }
            }}
        </button>
    }
}
