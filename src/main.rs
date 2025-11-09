mod api;
mod domain;

mod loader;
mod logger;

use crate::loader::parser::get_json_as_str;
use crate::domain::client::ClientRegistry;

fn main() {
    logger::init();

    log::info!("Logger initialized. Starting ClientRegistry.");

    let mut registry = ClientRegistry::new();
    let file_path = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/workflows.json";

    let json_str = get_json_as_str(file_path);

    match json_str {
        Some(json_str) => {
            log::info!("Loading from path: '{}'...", file_path);
            registry.update_clients(&json_str);
            registry.print_summary();
        }
        None => {
            log::error!("Error during loading of json file!");
        }
    }
}