#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct PlexConfig {
    pub token: String,
    pub url: String,
}

impl PlexConfig {
    pub fn get_playlists_url(&self) -> String {
        format!("{}/playlists?playlistType=audio&includeCollections=1&includeExternalMedia=1&includeAdvanced=1&includeMeta=1&{}", self.url, self.get_static_query_params())
    }

    pub fn get_static_query_params(&self) -> String {
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