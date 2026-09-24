// Veyra is a GUI application: never create a console window on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    veyra_lib::run()
}
