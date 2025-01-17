use crate::config::CONFIG;
use axum::{
    extract::Path,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
};
use reqwest::header::AUTHORIZATION;
use std::{
    fs::{self, File},
    io::Write,
    path::Path as StdPath,
};

/// Create a new sound from a text
pub async fn tts(text: String, filename: String) -> Result<(), String> {
    let url = "https://api.v7.unrealspeech.com/stream";
    let api_key = format!("Bearer {}", CONFIG.unrealspeech_token);

    let body = serde_json::json!({
        "Text": text,
        "VoiceId": "Will",
        "Bitrate": "192k",
        "Speed": "0",
        "Pitch": "0.92",
        "Codec": "libmp3lame",
    });

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error creating new audio: {}", e))?;

    if response.status().is_success() {
        let filepath = format!("{}/{}", CONFIG.audio_path, filename);
        if let Some(parent) = StdPath::new(&filepath).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        let mut file =
            File::create(&filepath).map_err(|e| format!("Failed to create file: {}", e))?;
        let content = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to get response bytes: {}", e))?;
        file.write_all(&content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    } else {
        Err(format!("Failed to fetch the audio: {}", response.status()))
    }
}

/// Axum endpoint which shows files
pub async fn show_file(
    Path(id): Path<String>,
) -> Result<(HeaderMap, Vec<u8>), (StatusCode, String)> {
    let index = id.find('.').unwrap_or(usize::MAX);
    let ext_name = if index != usize::MAX {
        &id[index + 1..]
    } else {
        "xxx"
    };

    let mut headers = HeaderMap::new();
    if ["mp3"].contains(&ext_name) {
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str("audio/mpeg").unwrap(),
        );
    }

    let file_name = format!("{}/{}", CONFIG.audio_path, id);
    let file_path = StdPath::new(&file_name);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    fs::read(file_path)
        .map(|file_content| (headers, file_content))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read file".to_string(),
            )
        })
}
