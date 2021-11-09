const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn get_xml_from_url(from_url: String) -> Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(from_url)?.text()?;
    Ok(body)
}

pub mod download {
    use std::{fs::File, path::PathBuf};

    use indicatif::{ProgressBar, ProgressStyle};
    use reqwest::blocking::Client;
    use transfer_progress::Transfer;

    #[derive(Debug)]
    pub enum DownloadError {
        GetRequestFailed(reqwest::Error),
        ContentLengthNotAvailable,
        IoError(std::io::Error),
    }

    pub fn download_with_progress(
        path: PathBuf,
        url: &str,
        download_name: Option<&str>,
    ) -> Result<(), DownloadError> {
        let client = Client::new();
        let res = client
            .get(url)
            .send()
            .map_err(|e| DownloadError::GetRequestFailed(e))?;
        let total_size = res
            .content_length()
            .ok_or(DownloadError::ContentLengthNotAvailable)?;

        // Indicatif setup
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
        .template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));

        if let Some(download_name) = download_name {
            pb.set_message(format!("Downloading {}", download_name));
        }

        // Download
        let file = File::create(path).map_err(|e| DownloadError::IoError(e))?;
        let transfer = Transfer::new(res, file);

        while !transfer.is_complete() {
            pb.set_position(transfer.transferred());
        }

        let (_reader, _writer) = transfer.finish().map_err(|e| DownloadError::IoError(e))?;

        let finish_message = match download_name {
            Some(name) => format!("Downloaded {}", name),
            None => "Downloaded".into(),
        };
        pb.finish_with_message(finish_message);

        return Ok(());
    }
}

mod plex {
    use crate::get_xml_from_url;
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
}

mod ui {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn start_spinner(msg: &'static str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(120);
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}"));
        pb.set_message(msg);
        pb
    }
}

mod tools {
    use std::io;

    use crate::download;

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
        DownloadError(download::DownloadError),
    }

    pub mod playlist_syncer {
        use crate::download::{self, DownloadError};
        use crate::ui::start_spinner;
        use crate::{plex::*, tools::plex_config::read_config};
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
            playlist_ref: &'a PlaylistOverview,
            config: &'a PlexConfig,
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

        pub fn perform_download(downloads: Vec<TrackDownload>) -> Result<(), DownloadError> {
            for download in downloads {
                let download_dir = download
                    .path
                    .parent()
                    .expect("Could not create download directory");

                create_dir_all(download_dir).map_err(|e| DownloadError::IoError(e))?;
                let download_result = download::download_with_progress(
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
            let playlists = playlists::fetch_all(&config);
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
    }

    pub mod plex_config {
        use super::*;
        use crate::plex::PlexConfig;
        use dirs::config_dir;
        use std::fs::{self, create_dir_all};
        use std::path::PathBuf;

        #[derive(Debug)]
        pub enum Error {
            SerializationError(toml::ser::Error),
            IoError(io::Error),
        }

        pub const TOOL: ToolDescription = ToolDescription {
            name: "plex-init",
            description: "Specify access to your Plex instance",
            execute_interactive: plex_config_interactive,
            is_active: || true,
        };

        fn get_config_path() -> PathBuf {
            let path = config_dir().unwrap().join("chiubi.cloud").join("conf.toml");
            path
        }

        pub fn is_config_existing() -> bool {
            get_config_path().as_path().is_file()
        }

        /// Reads the plex configuration from it's standard location
        pub fn read_config() -> Option<PlexConfig> {
            let config_contents =
                fs::read_to_string(get_config_path());

            if let Err(_) = config_contents {
                return None;
            }
            
            let config_contents = config_contents.unwrap();

            let config: Result<PlexConfig, toml::de::Error> = toml::from_str(&config_contents);

            match config {
                Ok(result) => Some(result),
                Err(_) => None,
            }
        }

        fn persist_config(config: &PlexConfig) -> Result<(), Error> {
            let config_contents =
                toml::to_string(config).map_err(|e| Error::SerializationError(e))?;
            let config_path = get_config_path();
            let config_dir = config_path.parent().unwrap();
            create_dir_all(config_dir).map_err(|e| Error::IoError(e))?;
            fs::write(&config_path, config_contents).map_err(|e| Error::IoError(e))?;
            Ok(())
        }

        pub fn plex_config_interactive() -> ToolResult {
            let old_config = read_config();
            // let old_config = old_config.unwrap_or_default();

            let mut input_token_builder = requestty::Question::input("Access Token");
            if let Some(default_token) =old_config.as_ref().and_then(|c| Some(c.token.clone())) {
                input_token_builder = input_token_builder.default(default_token);
            }

            let mut input_url_builder =  requestty::Question::input("Url");
            if let Some(default_url) = old_config.as_ref().and_then(|c| Some(c.url.clone())) {
                input_url_builder = input_url_builder.default(default_url);
            }

            let questions = vec![
                input_token_builder.build(),
                input_url_builder.build(),
            ];
            let answers = requestty::prompt(questions).expect("Could not interpret your answers");

            let config = PlexConfig {
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

            match persist_result {
                Ok(_) => {
                    println!("Config saved: {:?}", get_config_path());
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Could not persist config!");
                    eprintln!("Error: {:?}", e);
                    return Err(ToolError::ConfigError(e));
                }
            }
        }
    }

    mod exit {
        use super::{ToolDescription, ToolError, ToolResult};

        pub const TOOL: ToolDescription = ToolDescription {
            name: "exit",
            description: "Exit program",
            execute_interactive: execute,
            is_active: || true,
        };

        fn execute() -> ToolResult {
            Err(ToolError::Abort)
        }
    }

    pub mod main {
        use crate::VERSION;

        use super::*;

        use super::{exit, playlist_syncer, plex_config, ToolDescription};

        const MAIN_MENU_TOOLS: [ToolDescription; 3] =
            [plex_config::TOOL, playlist_syncer::TOOL, exit::TOOL];

        fn print_header() {
            println!(
                r"
              __    _       __    _       __                __
        _____/ /_  (_)_  __/ /_  (_)_____/ /___  __  ______/ /
       / ___/ __ \/ / / / / __ \/ // ___/ / __ \/ / / / __  / 
      / /__/ / / / / /_/ / /_/ / // /__/ / /_/ / /_/ / /_/ /  
      \___/_/ /_/_/\__,_/_.___/_(_)___/_/\____/\__,_/\__,_/                            
            "
            );
            println!("Version: {}", VERSION);
        }

        pub fn main_menu_interactive() {
            loop {
                let tool_entries = MAIN_MENU_TOOLS
                    .iter()
                    .filter(|t| {
                        let is_active = t.is_active;
                        is_active()
                    })
                    .map(|t| format!("{}: {}", t.name, t.description))
                    .collect::<Vec<_>>();

                print_header();

                let has_config = plex_config::is_config_existing();
                if has_config {
                    println!("Plex configuration: ✔");
                } else {
                    println!("Plex configuration: ❌");
                }

                let question = requestty::Question::select("Tools")
                    .choices(tool_entries)
                    .build();
                let answer = requestty::prompt_one(question).unwrap();
                let answer = answer
                    .as_list_item()
                    .expect("Could not process main menu item");
                let tool_index = answer.index;
                let tool = &MAIN_MENU_TOOLS[tool_index];
                let tool_function = tool.execute_interactive;
                let result = tool_function();

                if let Err(e) = result {
                    match e {
                        ToolError::Abort => break,
                        _ => println!("Error: {:?}", e),
                    }
                }
            }
        }
    }
}

fn main() {
    tools::main::main_menu_interactive();
}
