use plex::config::PlexConfig;
use plex::{Playlist, PlaylistOverview};
use crate::ui::start_spinner;

pub fn fetch_playlists(config: &PlexConfig) -> Vec<PlaylistOverview> {
    let spinner = start_spinner("Loading playlists");
    let playlists = plex::playlists::fetch_all(&config);
    spinner.finish_and_clear();
    return playlists;
}

pub fn select_playlist(playlists: &Vec<PlaylistOverview>) -> &PlaylistOverview {
    let question = requestty::Question::raw_select("Select a playlist")
        .choices(playlists.iter().map(|p| p.title.as_str()))
        .build();
    let answer = requestty::prompt_one(question).unwrap();
    let index = answer.as_list_item().unwrap().index;
    &playlists[index]
}

pub fn load_playlist_details(config: &PlexConfig, playlist_overview: &PlaylistOverview) -> Playlist {
    let spinner = start_spinner("Loading playlist information");
    let playlist = playlist_overview.into_detailed_playlist(config);
    spinner.finish_with_message(format!("Playlist information loaded"));
    playlist
}