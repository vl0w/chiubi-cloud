mod plex;
mod download;
mod ui;
mod tools;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    tools::main::main_menu_interactive();
}
