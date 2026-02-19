use leptos::prelude::*;
use north_ui::{Icon, IconKind};

#[derive(Default, Clone, Copy)]
pub enum TaskMetaItemVariant {
    #[default]
    Default,
    Info,
    Danger,
}

impl TaskMetaItemVariant {
    pub fn classes(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Info => "text-text-secondary",
            Self::Danger => "text-danger",
        }
    }
}

#[component]
pub fn TaskMetaItem(
    icon: IconKind,
    #[prop(default = None)] on_click: Option<Callback<()>>,
    #[prop(default = TaskMetaItemVariant::Default)] variant: TaskMetaItemVariant,
    #[prop(default = "")] class: &'static str,
    #[prop(into, default = String::new())] style: String,
    children: Children,
) -> impl IntoView {
    let variant_classes = variant.classes();
    let click_classes = if on_click.is_some() {
        "cursor-pointer transition-colors"
    } else {
        ""
    };
    view! {
        <span
            class=format!(
                "inline-flex items-center gap-1 \
                 {variant_classes} {click_classes} {class}"
            )
            style=style
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    ev.stop_propagation();
                    cb.run(());
                }
            }
        >
            <Icon kind=icon class="w-3 h-3" />
            {children()}
        </span>
    }
}
