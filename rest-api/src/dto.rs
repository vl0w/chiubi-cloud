use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlaylistOverviewDto {
    id: String,
    title: String,
    links: serde_json::Value,
}

impl From<plex::PlaylistOverview> for PlaylistOverviewDto {
    fn from(p: plex::PlaylistOverview) -> Self {
        let id = format!("{:x}", md5::compute(&p.title));
        Self {
            id: id.clone(),
            title: p.title,
            links: serde_json::json!({ "tracks": format!("/playlists/{}/tracks", id) }),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TrackDto {
    title: String,
    album: String,
    artist: String,
}

impl From<plex::Track> for TrackDto {
    fn from(t: plex::Track) -> Self {
        Self {
            title: t.title,
            album: t.album,
            artist: t.artist,
        }
    }
}
