use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlInputElement};

/// Set the `text_content` of an element by id
///
/// # Panics
/// Panics if no element with id `id` exists
#[allow(clippy::needless_pass_by_value)]
pub fn set_text(id: &str, content: String) {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .set_text_content(Some(&content));
}

/// Set the `text_content` of an element by id
///
/// # Panics
/// Panics if no element with id `id` exists
pub fn set_text_str(id: &str, content: &str) {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .set_text_content(Some(content));
}

/// Get the value of a textarea by id
///
/// # Panics
/// Panics if no element with id `id` exists
#[must_use]
pub fn get_input(id: &str) -> String {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value()
}

/// Shorthand for `web_sys::window().unwrap().document.unwrap()`
///
/// # Panics
/// Refer to `web_sys::Window` and `web_sys::Document`
#[must_use]
pub fn get_document() -> Document {
    window().unwrap().document().unwrap()
}
