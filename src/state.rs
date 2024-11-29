use std::sync::Arc;

use expo_push_notification_client::Expo;
use tokio_postgres::Client;

#[derive(Clone)]
/// State application shared through Axum
pub struct AppState {
    /// PostgreSQL client synced via Arc
    pub client: Arc<Client>,

    /// Expo connection
    pub expo: Arc<Expo>,
}
