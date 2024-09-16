use crate::errors::AppError;

use tokio_postgres::{Client, NoTls};

/// Setup database connection. Get variable `DATABASE_URL` from the environment.
pub async fn setup() -> Result<Client, AppError> {
    let database_url = &crate::config::CONFIG.database_url;

    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await.unwrap();

    // Spawn a new task to run the connection to the database
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("Database connection error: {}", e);
        }
    });

    Ok(client)
}
