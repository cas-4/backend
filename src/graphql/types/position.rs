use crate::{dates::GraphQLDate, graphql::types::jwt::Authentication, state::AppState};
use async_graphql::{Context, Enum, Object};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

use super::user::find_user;

#[derive(Enum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MovingActivity {
    // "Car" of the doc
    InVehicle,

    // Walking or running
    OnFoot,

    // Running
    Running,

    // Walking
    Walking,

    // Device is not moving
    Still,
}

impl<'a> FromSql<'a> for MovingActivity {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<MovingActivity, Box<dyn Error + Sync + Send>> {
        match std::str::from_utf8(raw)? {
            "InVehicle" => Ok(MovingActivity::InVehicle),
            "OnFoot" => Ok(MovingActivity::OnFoot),
            "Running" => Ok(MovingActivity::Running),
            "Walking" => Ok(MovingActivity::Walking),
            "Still" => Ok(MovingActivity::Still),
            other => Err(format!("Unknown variant: {}", other).into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "moving_activity"
    }
}

impl ToSql for MovingActivity {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut bytes::BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let value = match *self {
            MovingActivity::InVehicle => "InVehicle",
            MovingActivity::OnFoot => "OnFoot",
            MovingActivity::Running => "Running",
            MovingActivity::Walking => "Walking",
            MovingActivity::Still => "Still",
        };
        out.extend_from_slice(value.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "moving_activity"
    }

    to_sql_checked!();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: i32,
    pub user_id: i32,
    pub created_at: GraphQLDate,
    pub latitude: f64,
    pub longitude: f64,
    pub moving_activity: MovingActivity,
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

    async fn moving_activity(&self) -> MovingActivity {
        self.moving_activity
    }
}

pub async fn get_positions<'ctx>(
    ctx: &Context<'ctx>,
    user_id: Option<i32>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Option<Vec<Position>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(claims) => {
            let rows;
            let claim_user = find_user(client, claims.user_id)
                .await
                .expect("Should not be here");

            if claim_user.is_admin {
                match user_id {
                    Some(id) => {
                        rows = client.query("
                            SELECT id, user_id, created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions
                            WHERE user_id = $1
                            ORDER BY id DESC
                            LIMIT $2
                            OFFSET $3",
                            &[&id, &limit.unwrap_or(20), &offset.unwrap_or(0)]).await.unwrap();
                    }
                    None => {
                        rows = client.query("
                            SELECT id, user_id, created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions
                            ORDER BY id DESC
                            LIMIT $1
                            OFFSET $2",
                            &[&limit.unwrap_or(20), &offset.unwrap_or(0)]).await.unwrap();
                    }
                }
            } else {
                rows = client.query("
                    SELECT id, user_id, created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                    FROM positions
                    WHERE user_id = $1
                    ORDER BY id DESC
                    LIMIT $2
                    OFFSET $3",
                    &[&claim_user.id, &limit.unwrap_or(20), &offset.unwrap_or(0)]).await.unwrap();
            }

            let positions: Vec<Position> = rows
                .iter()
                .map(|row| Position {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created_at: GraphQLDate(Utc::now()),
                    latitude: row.get("latitude"),
                    longitude: row.get("longitude"),
                    moving_activity: row.get("activity"),
                })
                .collect();

            Ok(Some(positions))
        }
    }
}
