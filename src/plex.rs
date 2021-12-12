use crate::download::get_xml_from_url;
use quick_xml::de::from_str;
use sanitize_filename::sanitize;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct PlexConfig {
    pub token: String,
    pub url: String,
}

impl PlexConfig {
    fn get_playlists_url(&self) -> String {
        format!("{}/playlists?playlistType=audio&includeCollections=1&includeExternalMedia=1&includeAdvanced=1&includeMeta=1&{}", self.url, self.get_static_query_params())
    }

    fn get_static_query_params(&self) -> String {
        format!("X-Plex-Token={}&X-Plex-Product=Plex%20Web&X-Plex-Version=4.64.3&X-Plex-Client-Identifier=mg7p5uivc6f90wsoxu2asvad&X-Plex-Platform=Chrome&X-Plex-Platform-Version=92.0&X-Plex-Sync-Version=2&X-Plex-Features=external-media%2Cindirect-media&X-Plex-Model=hosted&X-Plex-Device=Windows&X-Plex-Device-Name=Chrome&X-Plex-Device-Screen-Resolution=2498x632%2C2560x1440&X-Plex-Language=en-GB&X-Plex-Drm=widevine&X-Plex-Text-Format=plain&X-Plex-Provider-Version=3.2", self.token)
    }
}

impl Default for PlexConfig {
    fn default() -> Self {
        Self {
            token: Default::default(),
            url: Default::default(),
        }
    }
}

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