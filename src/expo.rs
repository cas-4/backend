use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage};

use crate::errors::AppError;

/// Setup a new Expo API
pub fn setup(access_token: String) -> Expo {
    Expo::new(ExpoClientOptions {
        access_token: Some(access_token),
    })
}

/// Send notifications using Expo
pub async fn send(
    expo: Expo,
    tokens: Vec<String>,
    body: String,
    title: String,
) -> Result<(), AppError> {
    let expo_push_message = ExpoPushMessage::builder(tokens)
        .body(body)
        .title(title)
        .build()?;

    if expo
        .send_push_notifications(expo_push_message)
        .await
        .is_err()
    {
        return Err(AppError::BadRequest(
            "Expo Notifications sending".to_string(),
        ));
    }

    Ok(())
}
