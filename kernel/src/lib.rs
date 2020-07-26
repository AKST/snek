mod game_loop;

use mythos_web::game_loop::start_game_loop;
use wasm_bindgen::prelude::*;

use crate::game_loop::DemoGameLoop;

#[wasm_bindgen]
pub fn start() -> js_sys::Promise {
  start_game_loop::<DemoGameLoop>()
}
