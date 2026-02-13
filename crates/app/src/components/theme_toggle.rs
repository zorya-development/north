use leptos::prelude::*;
use north_ui::{Icon, IconKind};

#[cfg(feature = "hydrate")]
mod js {
    use leptos::wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = "
        export function set_theme(theme) { localStorage.setItem('north-theme', theme); }
    ")]
    extern "C" {
        pub fn set_theme(theme: &str);
    }
}

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (is_dark, set_is_dark) = signal(false);

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let doc = document();
            if let Some(el) = doc.document_element() {
                let dark = el.class_list().contains("dark");
                set_is_dark.set(dark);
            }
        });
    }

    let toggle = move |_| {
        let new_dark = !is_dark.get_untracked();
        set_is_dark.set(new_dark);

        #[cfg(feature = "hydrate")]
        {
            let doc = document();
            if let Some(el) = doc.document_element() {
                let cl = el.class_list();
                if new_dark {
                    let _ = cl.add_1("dark");
                } else {
                    let _ = cl.remove_1("dark");
                }
            }
            let theme = if new_dark { "dark" } else { "light" };
            js::set_theme(theme);
        }
    };

    view! {
        <button
            on:click=toggle
            class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm \
                   text-text-primary hover:bg-bg-tertiary transition-colors \
                   w-full"
            aria-label="Toggle theme"
        >
            <Show
                when=move || is_dark.get()
                fallback=|| view! {
                    <Icon kind=IconKind::Moon class="w-4 h-4 flex-shrink-0"/>
                    "Dark mode"
                }
            >
                <Icon kind=IconKind::Sun class="w-4 h-4 flex-shrink-0"/>
                "Light mode"
            </Show>
        </button>
    }
}
