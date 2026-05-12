use iidiff_core::diff::DiffResponse;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn diff(sequence_a: &str, sequence_b: &str) -> JsValue {
    serde_wasm_bindgen::to_value(&DiffResponse::build(sequence_a, sequence_b)).unwrap()
}
