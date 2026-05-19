use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn contract_version() -> String {
    "daily-game-runtime.v1".to_string()
}
