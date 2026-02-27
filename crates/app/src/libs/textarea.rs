use wasm_bindgen::JsCast;

/// Insert a newline at the cursor position in a textarea, set the DOM value
/// directly, position the cursor, then dispatch an `input` event so the
/// `on:input` handler syncs the signal and runs `auto_resize` with the
/// correct `scrollHeight`.
pub fn insert_newline_at_cursor(ta: &web_sys::HtmlTextAreaElement) {
    let start = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
    let end = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
    let cur = ta.value();

    let mut next = String::with_capacity(cur.len() + 1);
    next.push_str(&cur[..start]);
    next.push('\n');
    next.push_str(&cur[end..]);

    // 1. Set DOM value directly so scrollHeight is correct for auto_resize.
    ta.set_value(&next);

    // 2. Place cursor right after the inserted newline.
    let pos = (start + 1) as u32;
    let _ = ta.set_selection_start(Some(pos));
    let _ = ta.set_selection_end(Some(pos));

    // 3. Dispatch an input event so the on:input handler syncs the signal
    //    and runs auto_resize with the correct scrollHeight.
    if let Ok(event) = web_sys::Event::new("input") {
        let _ = ta
            .dyn_ref::<web_sys::EventTarget>()
            .map(|t| t.dispatch_event(&event));
    }
}
