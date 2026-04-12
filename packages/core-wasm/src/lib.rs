use wasm_bindgen::prelude::*;
use flexjson_core;

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, JsValue> {
    let result = flexjson_core::parse(input);
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
