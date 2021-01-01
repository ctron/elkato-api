#![recursion_limit = "512"]

mod app;
mod current;
mod data;

use wasm_bindgen::prelude::*;

pub const BASE_URL: &str = "https://elkato-elkato.apps.wonderful.iot-playground.org";

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();
    Ok(())
}
