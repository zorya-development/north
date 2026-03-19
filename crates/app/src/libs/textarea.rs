use wasm_bindgen::JsCast;

/// Convert a UTF-16 code-unit offset (as returned by JS `selectionStart`/`selectionEnd`)
/// to a Rust UTF-8 byte offset for safe string slicing.
fn utf16_offset_to_byte_offset(s: &str, utf16_offset: usize) -> usize {
    let mut utf16_count = 0;
    for (byte_idx, ch) in s.char_indices() {
        if utf16_count >= utf16_offset {
            return byte_idx;
        }
        utf16_count += ch.len_utf16();
    }
    s.len()
}

/// Insert a newline at the cursor position in a textarea, set the DOM value
/// directly, position the cursor, then dispatch an `input` event so the
/// `on:input` handler syncs the signal and runs `auto_resize` with the
/// correct `scrollHeight`.
pub fn insert_newline_at_cursor(ta: &web_sys::HtmlTextAreaElement) {
    let start_utf16 = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
    let end_utf16 = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
    let cur = ta.value();

    // selection_start/end return UTF-16 offsets, but Rust strings are UTF-8.
    // Convert UTF-16 offsets to byte offsets for correct slicing.
    let start = utf16_offset_to_byte_offset(&cur, start_utf16);
    let end = utf16_offset_to_byte_offset(&cur, end_utf16);

    let mut next = String::with_capacity(cur.len() + 1);
    next.push_str(&cur[..start]);
    next.push('\n');
    next.push_str(&cur[end..]);

    // 1. Set DOM value directly so scrollHeight is correct for auto_resize.
    ta.set_value(&next);

    // 2. Place cursor right after the inserted newline (UTF-16 offset).
    let pos = (start_utf16 + 1) as u32;
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
