mod dto;

use crate::dto::PlaylistOverviewDto;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::{routing::get, Json, Router};
use itertools::Itertools;
use plex::config::PlexConfig;
use serde_json::json;
use std::thread;
use sync_wrapper::SyncWrapper;

fn __playlists(config: PlexConfig) -> Vec<plex::PlaylistOverview> {
    thread::spawn(move || plex::playlists::fetch_all(&config))
        .join()
        .expect("Thread panicked")
}

async fn get_playlists(Query(params): Query<PlexConfig>) -> Json<Vec<serde_json::Value>> {
    let playlists = __playlists(params);

    let jsons = playlists
        .into_iter()
        .map(|p| PlaylistOverviewDto::from(p))
        .map(|p| json!(p))
        .collect::<Vec<serde_json::Value>>();
    Json(jsons)
}

async fn get_tracks_of_playlist(
    Query(config): Query<PlexConfig>,
    Path(playlist_id): Path<String>,
) -> Result<String, StatusCode> {
    // Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let playlists = __playlists(config.clone());

    println!("hi");
    let playlist = playlists
        .into_iter()
        .filter(|p| {
            let id = format!("{:x}", md5::compute(&p.title));
            id == playlist_id
        })
        .exactly_one()
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let playlist = thread::spawn(move || playlist.into_detailed_playlist(&config.clone()))
        .join()
        .expect("Thread panicked");

    let tracks = playlist
        .tracks
        .into_iter()
        .map(|t| format!("{};{};{}", t.artist, t.album, t.title))
        .collect_vec()
        .join("\n");
    let response = format!("artist;album;title\n{}", tracks);

    // let tracks = playlist
    //     .tracks
    //     .into_iter()
    //     .map(|t| TrackDto::from(t))
    //     .map(|t| json!(t))
    //     .collect_vec();

    // Ok(Json(tracks))
    Ok(response)
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new()
        .route("/playlists", get(get_playlists))
        //
        .route(
            "/playlists/:playlist_id/tracks",
            get(get_tracks_of_playlist),
        );
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
