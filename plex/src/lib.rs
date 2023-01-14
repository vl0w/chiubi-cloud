pub mod config;

use config::PlexConfig;
use quick_xml::de::from_str;
use sanitize_filename::sanitize;
use downloader::get_xml_from_url;

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename = "Part")]
struct XmlTrackMediaParts {
    key: String,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename = "Media")]
struct TrackMedia {
    #[serde(rename = "Part")]
    parts: Vec<XmlTrackMediaParts>,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename = "Track")]
pub struct Track {
    pub title: String,
    #[serde(rename = "parentTitle")]
    pub album: String,
    #[serde(rename = "grandparentTitle")]
    pub artist: String,

    #[serde(rename = "Media")]
    media: Vec<TrackMedia>,
}

impl Track {
    pub fn get_download_url(&self, config: &PlexConfig) -> String {
        assert!(self.media.len() == 1);
        assert!(self.media[0].parts.len() == 1);
        let parts_key = self.get_parts_key();
        format!(
            "{}{}?{}",
            config.url,
            parts_key,
            config.get_static_query_params()
        )
    }

    pub fn get_file_extension(&self) -> String {
        let parts_key = self.get_parts_key();
        let delimiter = parts_key.find(".").expect(
            format!(
                "Could not detect file extension for parts key {}",
                parts_key
            )
                .as_str(),
        );
        parts_key[delimiter..].into()
    }

    pub fn infer_file_name(&self) -> String {
        let raw_name = format!(
            "{} - {}{}",
            self.artist,
            self.title,
            self.get_file_extension()
        );
        sanitize(raw_name)
    }

    fn get_parts_key(&self) -> String {
        self.media[0].parts[0].key.clone()
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename = "Playlist")]
pub struct Playlist {
    pub title: String,

    #[serde(rename = "Track")]
    pub tracks: Vec<Track>,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename = "Playlist")]
pub struct PlaylistOverview {
    key: String,
    pub title: String,
    pub summary: String,
}

impl PlaylistOverview {
    pub fn get_playlist_url(&self, config: &PlexConfig) -> String {
        format!(
            "{}{}?{}",
            config.url,
            self.key,
            config.get_static_query_params()
        )
    }

    pub fn into_detailed_playlist(&self, config: &PlexConfig) -> Playlist {
        let url = self.get_playlist_url(config);
        let xml_str = get_xml_from_url(url).unwrap();
        let xml_str = xml_str.as_str();
        let playlist: Playlist = from_str(xml_str).unwrap();
        playlist
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename = "MediaContainer")]
struct XmlPlaylists {
    #[serde(rename = "Playlist")]
    pub playlists: Vec<PlaylistOverview>,
}

pub mod playlists {
    use super::*;

    pub fn fetch_all(config: &PlexConfig) -> Vec<PlaylistOverview> {
        let xml_str = get_xml_from_url(config.get_playlists_url()).unwrap();
        let xml_str = xml_str.as_str();
        let xml_playlists: XmlPlaylists = from_str(xml_str).unwrap();
        xml_playlists.playlists
    }
}