
#[macro_use]
extern crate dotenv_codegen;

mod app;
use wasm_logger;

use app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::start_app::<App>();
}
