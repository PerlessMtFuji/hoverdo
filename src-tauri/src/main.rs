// Hide the console window on Windows release builds, while keeping it for
// debug builds so we still see logs/panics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hoverdo_lib::run();
}
