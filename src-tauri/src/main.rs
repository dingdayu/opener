// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[macro_use]
mod macros;

use std::panic;

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let err_message = match panic_info.payload().downcast_ref::<&str>() {
            Some(msg) => *msg,
            None => "Unknown panic error",
        };

        let location = panic_info
            .location()
            .map_or("unknown location".to_string(), |l| {
                format!("{}:{}", l.file(), l.line())
            });

        log::error!("⚠️ Panic occurred at {}: {}", location, err_message);
    }));
}

fn main() {
    setup_panic_hook();
    opener_lib::run()
}
