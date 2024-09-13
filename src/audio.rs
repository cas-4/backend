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
    let filepath = format!("./assets/sounds/{}", filename);

    // Request JSON body
    let body = serde_json::json!({
        "Text": text,
        "VoiceId": "Will",
        "Bitrate": "192k",
        "Speed": "0",
        "Pitch": "0.92",
        "Codec": "libmp3lame",
    });

    // Send POST request
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, api_key)
        .json(&body)
        .send()
        .await
        .unwrap();

    // Check for successful response
    if response.status().is_success() {
        let mut file = File::create(filepath).unwrap();
        let content = response.bytes().await.unwrap();
        let _ = file.write_all(&content);
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

    let mut ext_name = "xxx";
    if index != usize::MAX {
        ext_name = &id[index + 1..];
    }

    let mut headers = HeaderMap::new();

    if ["mp3"].contains(&ext_name) {
        let content_type = "audio/mpeg";
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str(content_type).unwrap(),
        );
    }

    let file_name = format!("./assets/sounds/{}", id);
    let file_path = StdPath::new(&file_name);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    // Read the file and return its content
    match fs::read(file_path) {
        Ok(file_content) => Ok((headers, file_content)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read file".to_string(),
        )),
    }
}
