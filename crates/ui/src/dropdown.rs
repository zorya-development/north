use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[component]
pub fn DropdownMenu(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    trigger: Children,
    children: Children,
) -> impl IntoView {
    let menu_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        if open.get() {
            let handle =
                window_event_listener(leptos::ev::click, move |ev: leptos::ev::MouseEvent| {
                    if let Some(menu) = menu_ref.get() {
                        let target = ev.target();
                        let is_inside = target
                            .and_then(|t| t.dyn_into::<leptos::web_sys::Node>().ok())
                            .map(|node| menu.contains(Some(&node)))
                            .unwrap_or(false);
                        if !is_inside {
                            set_open.set(false);
                        }
                    }
                });
            on_cleanup(move || drop(handle));
        }
    });

    let menu_children = children();

    view! {
        <div node_ref=menu_ref class="relative">
            {trigger()}
            <div
                class="absolute right-0 top-full mt-1 z-50 min-w-[140px] \
                        bg-bg-secondary border border-border rounded-lg \
                        shadow-lg py-1"
                style:display=move || {
                    if open.get() { "block" } else { "none" }
                }
            >
                {menu_children}
            </div>
        </div>
    }
}

#[component]
pub fn DropdownItem<F>(
    label: &'static str,
    on_click: F,
    #[prop(optional)] danger: bool,
) -> impl IntoView
where
    F: Fn() + Send + Sync + 'static,
{
    let text_class = if danger {
        "w-full text-left px-3 py-1.5 text-sm text-accent \
         hover:bg-bg-tertiary transition-colors"
    } else {
        "w-full text-left px-3 py-1.5 text-sm text-text-primary \
         hover:bg-bg-tertiary transition-colors"
    };

    let handle_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        on_click();
    };

    view! {
        <button class=text_class on:click=handle_click>
            {label}
        </button>
    }
}
