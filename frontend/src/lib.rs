use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlTextAreaElement};

/// # Panics
/// Panics if no element with id `id` exists
pub fn set_text(id: &str, content: &str) {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .set_text_content(Some(content));
}

/// # Panics
/// Panics if no element with id `id` exists
#[must_use]
pub fn get_textarea(id: &str) -> String {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .unchecked_into::<HtmlTextAreaElement>()
        .value()
}

/// # Panics
/// Refer to `web_sys::Window` and `web_sys::Document`
#[must_use]
pub fn get_document() -> Document {
    window().unwrap().document().unwrap()
}
