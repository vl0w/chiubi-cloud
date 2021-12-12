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

pub fn get_xml_from_url(from_url: String) -> Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(from_url)?.text()?;
    Ok(body)
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