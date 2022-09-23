
#[macro_use]
extern crate dotenv_codegen;

use wasm_logger;
mod app;
use app::App;
mod components;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::start_app::<App>();
}
