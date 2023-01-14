mod common;
pub mod exit;
pub mod main;
pub mod playlist_export;
pub mod playlist_syncer;
pub mod plex_config;
pub mod print_config;

type ToolResult = Result<(), ToolError>;

pub struct ToolDescription {
    name: &'static str,
    description: &'static str,
    execute_interactive: fn() -> ToolResult,
    is_active: fn() -> bool,
}

#[derive(Debug)]
pub enum ToolError {
    Abort,
    ConfigError(plex_config::Error),
    NoPlexConfig,
    DownloadError(downloader::Error),
}

fn get_config_path() -> std::path::PathBuf {
    let path = dirs::config_dir()
        .unwrap()
        .join("chiubi.cloud")
        .join("conf.toml");
    path
}

pub fn is_config_existing() -> bool {
    get_config_path().as_path().is_file()
}

/// Reads the plex configuration from it's standard location
pub fn read_config() -> Option<plex::config::PlexConfig> {
    let config_content = std::fs::read_to_string(get_config_path());

    if let Err(_) = config_content {
        return None;
    }

    let config_contents = config_content.unwrap();

    let config: Result<plex::config::PlexConfig, toml::de::Error> =
        toml::from_str(&config_contents);

    match config {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}
