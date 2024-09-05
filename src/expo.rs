use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage, ValidationError};

/// Connection to an Expo client
static mut EXPO_CONNECTION: Option<Expo> = None;

/// Setup a new Expo API
pub fn setup(access_token: String) {
    unsafe {
        EXPO_CONNECTION = Some(Expo::new(ExpoClientOptions {
            access_token: Some(access_token),
        }))
    }
}

/// Send notifications using Expo
pub async fn send(tokens: Vec<String>, body: String, title: String) -> Result<(), ValidationError> {
    let expo = unsafe {
        EXPO_CONNECTION
            .clone()
            .expect("You need to call `setup()` first")
    };

    let expo_push_message = ExpoPushMessage::builder(tokens)
        .body(body)
        .title(title)
        .build()?;

    let _ = expo.send_push_notifications(expo_push_message).await;

    Ok(())
}
