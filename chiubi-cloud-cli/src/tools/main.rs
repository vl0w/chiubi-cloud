use crate::VERSION;

use super::*;

use super::{exit, playlist_syncer, plex_config, ToolDescription};

const MAIN_MENU_TOOLS: [ToolDescription; 4] =
    [plex_config::TOOL, playlist_syncer::TOOL, playlist_export::TOOL, exit::TOOL];

fn print_header() {
    println!(
        r"
              __    _       __    _       __                __
        _____/ /_  (_)_  __/ /_  (_)_____/ /___  __  ______/ /
       / ___/ __ \/ / / / / __ \/ // ___/ / __ \/ / / / __  /
      / /__/ / / / / /_/ / /_/ / // /__/ / /_/ / /_/ / /_/ /
      \___/_/ /_/_/\__,_/_.___/_(_)___/_/\____/\__,_/\__,_/
            "
    );
    println!("Version: {}", VERSION);
}

pub fn main_menu_interactive() {
    loop {
        let tool_entries = MAIN_MENU_TOOLS
            .iter()
            .filter(|t| {
                let is_active = t.is_active;
                is_active()
            })
            .map(|t| format!("{}: {}", t.name, t.description))
            .collect::<Vec<_>>();

        print_header();

        let has_config = is_config_existing();
        if has_config {
            println!("Plex configuration: ✔");
        } else {
            println!("Plex configuration: ❌");
        }

        let question = requestty::Question::select("Tools")
            .choices(tool_entries)
            .build();
        let answer = requestty::prompt_one(question).unwrap();
        let answer = answer
            .as_list_item()
            .expect("Could not process main menu item");
        let tool_index = answer.index;
        let tool = &MAIN_MENU_TOOLS[tool_index];
        let tool_function = tool.execute_interactive;
        let result = tool_function();

        if let Err(e) = result {
            match e {
                ToolError::Abort => break,
                _ => println!("Error: {:?}", e),
            }
        }
    }
}