//! Bare - Entrypoint
//!
//! Starter applikasjonen.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bare_lib::run();
}
