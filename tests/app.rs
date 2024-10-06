use futures::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn run_test() {
    assert_eq!(1, 1);
}

#[wasm_bindgen_test]
fn web_test() {
    assert_eq!(1, 1);
}