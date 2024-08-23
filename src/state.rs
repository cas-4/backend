use std::sync::Arc;

use tokio_postgres::Client;

#[derive(Clone)]
/// State application shared through Axum
pub struct AppState {
    /// PostgreSQL client synced via Arc
    pub client: Arc<Client>,
}
