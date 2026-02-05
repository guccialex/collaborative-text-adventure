use leptos::web_sys;

pub fn scroll_to_segment(id: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(&format!("segment-{}", id)) {
            let _ = el.scroll_into_view_with_bool(true);
        }
    }
}
