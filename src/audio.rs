use crate::config::CONFIG;
use reqwest::header::AUTHORIZATION;

/// Create a new sound from a text
pub async fn tts(text: &String) -> Result<bytes::Bytes, String> {
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
        let content = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to get response bytes: {}", e))?;

        Ok(content)
    } else {
        Err(format!("Failed to fetch the audio: {}", response.status()))
    }
}
