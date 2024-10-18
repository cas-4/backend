use crate::{
    errors::AppError,
    graphql::types::{jwt::Authentication, user::find_user},
    state::AppState,
};
use async_graphql::{Context, Enum, FieldResult, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::{
    types::{to_sql_checked, FromSql, IsNull, ToSql, Type},
    Client,
};

#[derive(Enum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
/// Enumeration which refers to the kind of moving activity
pub enum MovingActivity {
    // "Car" of the doc
    InVehicle,

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

/// Find a position with user_id = `id` using the PostgreSQL `client`
pub async fn find_user_position(client: &Client, id: i32) -> Result<Position, AppError> {
    let rows = client
        .query(
            "SELECT id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
            FROM positions
            WHERE user_id = $1",
            &[&id],
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

    if positions.len() == 1 {
        Ok(positions[0].clone())
    } else {
        Err(AppError::NotFound("Position".to_string()))
    }
}

pub mod query {
    use super::*;

    /// Get positions from the database for each user.
    /// It is restricted to only admin users.
    pub async fn get_positions<'ctx>(
        ctx: &Context<'ctx>,

        // Optional filter by moving activity
        moving_activity: Option<MovingActivity>,

        // Optional limit results
        limit: Option<i64>,

        // Optional offset results. It should be used with limit field.
        offset: Option<i64>,
    ) -> Result<Option<Vec<Position>>, AppError> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;
        let auth: &Authentication = ctx.data()?;
        match auth {
            Authentication::NotLogged => Err(AppError::Unauthorized),
            Authentication::Logged(claims) => {
                let limit = limit.unwrap_or(20);
                let offset = offset.unwrap_or(0);

                let claim_user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                if !claim_user.is_admin {
                    return Err(AppError::Unauthorized);
                }

                let rows = client
                        .query("
                            SELECT id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions
                            LIMIT $1
                            OFFSET $2
                            ",
                            &[&limit, &offset],
                        )
                        .await?;

                let mapped_positions = rows.iter().map(|row| Position {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created_at: row.get::<_, f64>("created_at") as i64,
                    latitude: row.get("latitude"),
                    longitude: row.get("longitude"),
                    moving_activity: row.get("activity"),
                });

                let positions: Vec<Position>;
                if let Some(activity) = moving_activity {
                    positions = mapped_positions
                        .filter(|x| x.moving_activity == activity)
                        .collect();
                } else {
                    positions = mapped_positions.collect();
                }

                Ok(Some(positions))
            }
        }
    }
}

pub mod mutations {
    use super::*;

    /// Create a new position for a logged user. If a position already exists, just edit that
    /// position.
    pub async fn new_position<'ctx>(
        ctx: &Context<'ctx>,
        input: PositionInput,
    ) -> FieldResult<Position> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data()?;
        match auth {
            Authentication::NotLogged => {
                Err(AppError::NotFound("Can't find the owner".to_string()).into())
            }
            Authentication::Logged(claims) => {
                let rows = if find_user_position(client, claims.user_id).await.is_ok() {
                    client.query(
                        "UPDATE positions SET
                        location = ST_SetSRID(ST_MakePoint($1, $2), 4326),
                        activity = $3
                        WHERE user_id = $4
                        RETURNING id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                        ",
                        &[
                            &input.longitude,
                            &input.latitude,
                            &input.moving_activity,
                            &claims.user_id,
                        ],
                    )
                    .await?
                } else {
                    client.query(
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
                    .await?
                };

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
