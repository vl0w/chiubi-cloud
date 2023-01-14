pub mod playlist_syncer;
pub mod plex_config;
pub mod exit;
pub mod main;

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
