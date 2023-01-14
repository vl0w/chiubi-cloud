use crate::ui::start_spinner;
use crate::{tools::plex_config::read_config};
use dirs::{audio_dir, download_dir, home_dir};
use sanitize_filename::sanitize;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

use super::plex_config::is_config_existing;
use super::{ToolDescription, ToolError};

pub const TOOL: ToolDescription = ToolDescription {
    name: "sync-playlist",
    description: "Synchronize a Plex music playlist to a folder",
    execute_interactive: sync_playlist_interactive,
    is_active: is_config_existing,
};

#[derive(Debug)]
pub struct SyncOptions<'a> {
    path: &'a Path,
    playlist_ref: &'a plex::PlaylistOverview,
    config: &'a plex::config::PlexConfig,
}

pub struct TrackDownload {
    url: String,
    path: PathBuf,
    file_name: String,
}

pub fn prepare_playlist_sync(options: SyncOptions) -> Vec<TrackDownload> {
    let config = options.config;

    let spinner = start_spinner("Loading playlist information");
    let tracks = options.playlist_ref.into_detailed_playlist(config).tracks;
    spinner.finish_with_message(format!(
        "Playlist information loaded, got {} tracks to synchronize",
        tracks.len()
    ));

    let existing_files = match fs::read_dir(options.path) {
        Ok(dir) => dir
            .map(|p| {
                p.expect("Could not read sync dir entry")
                    .file_name()
                    .into_string()
                    .expect("Error collecting existing files")
            })
            .collect::<Vec<String>>(),
        Err(_) => vec![],
    };

    let file_names = tracks
        .iter()
        .map(|t| t.infer_file_name())
        .collect::<Vec<_>>();

    let tracks_to_sync: Vec<TrackDownload> = tracks
        .into_iter()
        .zip(file_names)
        .filter(|(_, file_name)| !existing_files.contains(&file_name))
        .map(|(t, file_name)| TrackDownload {
            url: t.get_download_url(config),
            path: options.path.join(file_name.clone()),
            file_name,
        })
        .collect::<Vec<_>>();

    tracks_to_sync
}

pub fn perform_download(downloads: Vec<TrackDownload>) -> Result<(), downloader::Error> {
    for download in downloads {
        let download_dir = download
            .path
            .parent()
            .expect("Could not create download directory");

        create_dir_all(download_dir).map_err(|e| downloader::Error::IoError(e))?;
        let download_result = downloader::download_with_progress(
            download.path,
            download.url.as_str(),
            Some(download.file_name.as_str()),
        );

        if let Err(e) = download_result {
            return Err(e);
        }
    }

    Ok(())
}

fn default_playlist_sync_folder() -> Option<PathBuf> {
    audio_dir()
        .or_else(|| download_dir())
        .or_else(|| home_dir())
}

pub fn sync_playlist_interactive() -> Result<(), ToolError> {
    let config = read_config().ok_or(ToolError::NoPlexConfig)?;

    let spinner = start_spinner("Loading playlists");
    let playlists = plex::playlists::fetch_all(&config);
    spinner.finish_and_clear();

    // Select playlist
    let question = requestty::Question::raw_select("Select a playlist")
        .choices(playlists.iter().map(|p| p.title.as_str()))
        .build();
    let answer = requestty::prompt_one(question).unwrap();
    let index = answer.as_list_item().unwrap().index;
    let selected_playlist = &playlists[index];

    // Destination folder
    let folder = sanitize(selected_playlist.title.clone());
    let default_path = default_playlist_sync_folder().unwrap_or_default();
    let default_path = default_path.join(folder);
    let default_path = default_path.to_str().unwrap();
    let question = requestty::Question::input("Where to synchronize to?")
        .default(default_path)
        .build();
    let sync_path = requestty::prompt_one(question);
    let sync_path = sync_path.unwrap();
    let sync_path = sync_path.as_string();
    let sync_path = sync_path.expect("No sync path provided");
    let sync_path = Path::new(sync_path);

    // Confirmation
    let options = SyncOptions {
        path: sync_path,
        playlist_ref: selected_playlist,
        config: &config,
    };

    let downloads = prepare_playlist_sync(options);
    println!("{} tracks need to be downloaded", downloads.len());

    if downloads.len() == 0 {
        return Ok(());
    }

    let question = requestty::Question::confirm("Continue?").build();
    let answer = requestty::prompt_one(question)
        .unwrap()
        .as_bool()
        .unwrap_or(false);

    if answer {
        perform_download(downloads).map_err(|e| ToolError::DownloadError(e))?;
    }

    Ok(())
}
