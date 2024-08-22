use crate::{dates::GraphQLDate, state::AppState};
use async_graphql::{Context, Object};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::jwt::Authentication;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: i32,
    pub user_id: i32,
    pub created_at: GraphQLDate,
    pub latitude: f64,
    pub longitude: f64,
}

#[Object]
impl Position {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn user_id(&self) -> i32 {
        self.user_id
    }

    async fn created_at(&self) -> GraphQLDate {
        self.created_at.clone()
    }

    async fn latitude(&self) -> f64 {
        self.latitude
    }

    async fn longitude(&self) -> f64 {
        self.longitude
    }
}

pub async fn get_positions<'ctx>(
    ctx: &Context<'ctx>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Option<Vec<Position>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(claims) => {
            let rows = client.query("
                SELECT id, user_id, created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude
                FROM positions
                WHERE user_id = $1
                ORDER BY id DESC
                LIMIT $2
                OFFSET $3",
                &[&claims.user_id, &limit.unwrap_or(20), &offset.unwrap_or(0)]).await.unwrap();

            let positions: Vec<Position> = rows
                .iter()
                .map(|row| Position {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created_at: GraphQLDate(Utc::now()),
                    latitude: row.get("latitude"),
                    longitude: row.get("longitude"),
                })
                .collect();

            Ok(Some(positions))
        }
    }
}
