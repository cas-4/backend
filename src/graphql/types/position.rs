use crate::{graphql::types::jwt::Authentication, state::AppState};
use async_graphql::{Context, Enum, FieldResult, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

use super::user::find_user;

#[derive(Enum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
/// Enumeration which refers to the kind of moving activity
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

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
/// Position struct
pub struct Position {
    pub id: i32,
    pub user_id: i32,
    pub created_at: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub moving_activity: MovingActivity,
}

#[derive(InputObject)]
/// Position input struct
pub struct PositionInput {
    pub latitude: f64,
    pub longitude: f64,
    pub moving_activity: MovingActivity,
}

pub mod query {
    use super::*;

    /// Get positions from the database
    pub async fn get_positions<'ctx>(
        ctx: &Context<'ctx>,

        // Optional filter by user id. If not defined returns only available positions:
        // If claimed user is admin returns everything, otherwise only positions linked to that user.
        user_id: Option<i32>,

        // Optional limit results
        limit: Option<i64>,

        // Optional offset results. It should be used with limit field.
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
                            SELECT id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions
                            WHERE user_id = $1
                            ORDER BY id DESC
                            LIMIT $2
                            OFFSET $3",
                            &[&id, &limit.unwrap_or(20), &offset.unwrap_or(0)]).await.unwrap();
                        }
                        None => {
                            rows = client.query("
                            SELECT id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions
                            ORDER BY id DESC
                            LIMIT $1
                            OFFSET $2",
                            &[&limit.unwrap_or(20), &offset.unwrap_or(0)]).await.unwrap();
                        }
                    }
                } else {
                    rows = client.query("
                    SELECT id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
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
                        created_at: row.get::<_, f64>("created_at") as i64,
                        latitude: row.get("latitude"),
                        longitude: row.get("longitude"),
                        moving_activity: row.get("activity"),
                    })
                    .collect();

                Ok(Some(positions))
            }
        }
    }

    /// Get last positions from the database for each user.
    /// It is restricted to only admin users.
    pub async fn last_positions<'ctx>(
        ctx: &Context<'ctx>,

        // Optional filter by moving activity
        moving_activity: Option<MovingActivity>,
    ) -> Result<Option<Vec<Position>>, String> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;
        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err("Unauthorized".to_string()),
            Authentication::Logged(claims) => {
                let claim_user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                if !claim_user.is_admin {
                    return Err("Unauthorized".to_string());
                }

                let rows = client
                        .query(
                            "SELECT DISTINCT ON (user_id)
                                id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions ORDER BY user_id, created_at DESC",
                            &[],
                        )
                        .await
                        .unwrap();

                let positions: Vec<Position> = match moving_activity {
                    Some(activity) => rows
                        .iter()
                        .map(|row| Position {
                            id: row.get("id"),
                            user_id: row.get("user_id"),
                            created_at: row.get::<_, f64>("created_at") as i64,
                            latitude: row.get("latitude"),
                            longitude: row.get("longitude"),
                            moving_activity: row.get("activity"),
                        })
                        .filter(|x| x.moving_activity == activity)
                        .collect(),
                    None => rows
                        .iter()
                        .map(|row| Position {
                            id: row.get("id"),
                            user_id: row.get("user_id"),
                            created_at: row.get::<_, f64>("created_at") as i64,
                            latitude: row.get("latitude"),
                            longitude: row.get("longitude"),
                            moving_activity: row.get("activity"),
                        })
                        .collect(),
                };

                Ok(Some(positions))
            }
        }
    }
}

pub mod mutations {
    use super::*;

    /// Create a new position for a logged user
    pub async fn new_position<'ctx>(
        ctx: &Context<'ctx>,
        input: PositionInput,
    ) -> FieldResult<Position> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err(async_graphql::Error::new("Can't find the owner")),
            Authentication::Logged(claims) => {
                let rows = client
                    .query(
                        "INSERT INTO positions (user_id, location, activity)
                        VALUES (
                            $1,
                            ST_SetSRID(ST_MakePoint($2, $3), 4326),
                            $4
                        )
                        RETURNING id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                        ",
                        &[
                            &claims.user_id,
                            &input.longitude,
                            &input.latitude,
                            &input.moving_activity,
                        ],
                    )
                    .await
                    .unwrap();

                let positions: Vec<Position> = rows
                    .iter()
                    .map(|row| Position {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        latitude: row.get("latitude"),
                        longitude: row.get("longitude"),
                        moving_activity: row.get("activity"),
                    })
                    .collect();

                Ok(positions[0].clone())
            }
        }
    }
}
