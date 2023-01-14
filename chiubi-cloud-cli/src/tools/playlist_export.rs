use crate::tools::common;
use super::{ToolDescription, ToolError};

pub const TOOL: ToolDescription = ToolDescription {
    name: "export-playlist",
    description: "Export playlist to CSV",
    execute_interactive: playlist_export_interactive,
    is_active: super::is_config_existing,
};

fn playlist_export_interactive() -> Result<(), ToolError> {
    let config = super::read_config().ok_or(ToolError::NoPlexConfig)?;

    let playlists = common::fetch_playlists(&config);
    let playlist = common::select_playlist(&playlists);
    let playlist = common::load_playlist_details(&config, playlist);

    println!("artist;album;title");
    playlist.tracks.iter().for_each(|track| {
        println!("{};{};{}", track.artist, track.album, track.title);
    });

    Ok(())
}