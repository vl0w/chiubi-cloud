use std::fs::{self, create_dir_all};
use super::{ToolDescription, ToolResult, ToolError};

#[derive(Debug)]
pub enum Error {
    SerializationError(toml::ser::Error),
    IoError(std::io::Error),
}

pub const TOOL: ToolDescription = ToolDescription {
    name: "plex-init",
    description: "Specify access to your Plex instance",
    execute_interactive: plex_config_interactive,
    is_active: || true,
};

fn persist_config(config: &plex::config::PlexConfig) -> Result<(), Error> {
    let config_contents =
        toml::to_string(config).map_err(|e| Error::SerializationError(e))?;
    let config_path = super::get_config_path();
    let config_dir = config_path.parent().unwrap();
    create_dir_all(config_dir).map_err(|e| Error::IoError(e))?;
    fs::write(&config_path, config_contents).map_err(|e| Error::IoError(e))?;
    Ok(())
}

fn plex_config_interactive() -> ToolResult {
    let old_config = super::read_config();

    let mut input_token_builder = requestty::Question::input("Access Token");
    if let Some(default_token) = old_config.as_ref().and_then(|c| Some(c.token.clone())) {
        input_token_builder = input_token_builder.default(default_token);
    }

    let mut input_url_builder = requestty::Question::input("Url");
    if let Some(default_url) = old_config.as_ref().and_then(|c| Some(c.url.clone())) {
        input_url_builder = input_url_builder.default(default_url);
    }

    let questions = vec![
        input_token_builder.build(),
        input_url_builder.build(),
    ];
    let answers = requestty::prompt(questions).expect("Could not interpret your answers");

    let config = plex::config::PlexConfig {
        token: answers
            .get_key_value("Access Token")
            .unwrap()
            .1
            .as_string()
            .unwrap()
            .into(),
        url: answers
            .get_key_value("Url")
            .unwrap()
            .1
            .as_string()
            .unwrap()
            .into(),
    };

    let persist_result = persist_config(&config);

    return match persist_result {
        Ok(_) => {
            println!("Config saved: {:?}", super::get_config_path());
            Ok(())
        }
        Err(e) => {
            eprintln!("Could not persist config!");
            eprintln!("Error: {:?}", e);
            Err(ToolError::ConfigError(e))
        }
    };
}