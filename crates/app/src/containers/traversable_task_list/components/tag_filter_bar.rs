use leptos::prelude::*;

use crate::atoms::{Text, TextColor, TextVariant};

#[component]
pub fn TagFilterBar(
    available_tags: Memo<Vec<(String, String)>>,
    active_tag_names: RwSignal<Vec<String>>,
    on_toggle: Callback<String>,
) -> impl IntoView {
    let has_tags = Memo::new(move |_| !available_tags.get().is_empty());

    view! {
        <Show when=move || has_tags.get()>
            <div
                data-testid="tag-filter-bar"
                class="flex flex-wrap items-center gap-2 px-1 py-1.5"
            >
                <Text variant=TextVariant::BodySm color=TextColor::Secondary>
                    "Filter:"
                </Text>
                <For
                    each=move || available_tags.get()
                    key=|(name, _)| name.clone()
                    children=move |(name, _color)| {
                        let name_for_click = name.clone();
                        let name_for_active = name.clone();
                        let display = format!("#{name}");
                        let on_toggle = on_toggle;

                        let is_active = Memo::new(move |_| {
                            active_tag_names.get().contains(&name_for_active)
                        });

                        view! {
                            <button
                                data-testid="tag-filter-button"
                                data-active=move || is_active.get().to_string()
                                class=move || {
                                    if is_active.get() {
                                        "text-xs text-accent cursor-pointer \
                                         transition-colors"
                                    } else {
                                        "text-xs text-text-tertiary \
                                         hover:text-text-secondary \
                                         cursor-pointer transition-colors"
                                    }
                                }
                                on:click=move |_| {
                                    on_toggle.run(name_for_click.clone());
                                }
                            >
                                {display.clone()}
                            </button>
                        }
                    }
                />
            </div>
        </Show>
    }
}
