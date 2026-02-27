use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[component]
pub fn Popover(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    trigger: Children,
    children: Children,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let trigger_ref = NodeRef::<leptos::html::Div>::new();
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    let panel = children();

    Effect::new(move |_| {
        if !open.get() {
            // Reset visibility for next open
            if let Some(panel_el) = panel_ref.get() {
                let panel_node: &web_sys::Node = &panel_el;
                let panel_ws: &web_sys::HtmlElement = panel_node.unchecked_ref();
                let _ = panel_ws.style().set_property("visibility", "hidden");
            }
            return;
        }
        let Some(trigger_el) = trigger_ref.get() else {
            return;
        };
        let Some(panel_el) = panel_ref.get() else {
            return;
        };

        let Some(win) = web_sys::window() else {
            return;
        };
        let vw = win
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(1024.0);
        let vh = win
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(768.0);

        // Access web_sys methods via Node → JsCast
        let trigger_node: &web_sys::Node = &trigger_el;
        let trigger_ws: &web_sys::Element = trigger_node.unchecked_ref();
        let panel_node: &web_sys::Node = &panel_el;
        let panel_ws: &web_sys::HtmlElement = panel_node.unchecked_ref();

        let tr = trigger_ws.get_bounding_client_rect();
        let pw = panel_ws.offset_width() as f64;
        let ph = panel_ws.offset_height() as f64;

        let gap = 4.0;
        let margin = 8.0;

        // Vertical: prefer below trigger, flip above if needed
        let top = if tr.bottom() + gap + ph <= vh - margin {
            tr.bottom() + gap
        } else if tr.top() - gap - ph >= margin {
            tr.top() - gap - ph
        } else {
            (vh - ph - margin).max(margin)
        };

        // Horizontal: prefer left-aligned, shift left if overflows
        let left = if tr.left() + pw <= vw - margin {
            tr.left()
        } else {
            (tr.right() - pw).max(margin)
        };

        let style = panel_ws.style();
        let _ = style.set_property("top", &format!("{top}px"));
        let _ = style.set_property("left", &format!("{left}px"));
        // Position set — reveal the panel
        let _ = style.remove_property("visibility");
    });

    view! {
        <div node_ref=trigger_ref class=format!("relative inline-flex {class}")>
            {trigger()}
            <div
                class="fixed inset-0 z-40"
                style:display=move || {
                    if open.get() { "block" } else { "none" }
                }
                on:click=move |_| set_open.set(false)
            />
            <div
                node_ref=panel_ref
                class="fixed z-50 rounded-xl shadow-lg"
                style="visibility: hidden; background-color: var(--bg-secondary); \
                       border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);"
                style:display=move || {
                    if open.get() { "block" } else { "none" }
                }
            >
                {panel}
            </div>
        </div>
    }
}
