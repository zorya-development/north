use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct SuggestionItem {
    pub name: String,
    pub color: String,
}

#[component]
pub fn AutocompleteDropdown(
    items: Vec<SuggestionItem>,
    highlighted: ReadSignal<usize>,
    on_select: Callback<String>,
) -> impl IntoView {
    if items.is_empty() {
        return view! { <div class="hidden"/> }.into_any();
    }

    view! {
        <div class="absolute top-full z-50 mt-1 bg-bg-secondary border \
                    border-border/60 rounded-xl shadow-lg p-1 w-[200px] \
                    max-h-[200px] overflow-y-auto">
            {items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let name = item.name.clone();
                    let color = item.color.clone();
                    let select_name = name.clone();
                    view! {
                        <button
                            class=move || {
                                let base = "w-full text-left px-3 py-1.5 text-sm \
                                            text-text-primary rounded transition-colors \
                                            flex items-center gap-2";
                                if highlighted.get() == i {
                                    format!("{base} bg-bg-tertiary")
                                } else {
                                    format!("{base} hover:bg-bg-tertiary")
                                }
                            }
                            on:mousedown=move |ev| {
                                ev.prevent_default();
                                on_select.run(select_name.clone());
                            }
                        >
                            <span
                                class="w-2.5 h-2.5 rounded-full flex-shrink-0"
                                style=format!("background-color: {}", color)
                            />
                            <span class="flex-1">{name}</span>
                        </button>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
    .into_any()
}
