use crate::{dates::GraphQLDate, graphql::types::jwt::Authentication, state::AppState};
use async_graphql::{Context, Enum, InputObject, SimpleObject};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

#[derive(Enum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
/// Enumeration which refers to the level of alert
pub enum LevelAlert {
    // User in the AREA
    One,

    // User in the AREA OR < 1km distance
    Two,

    // User in the AREA OR < 2km distance
    Three,
}

impl<'a> FromSql<'a> for LevelAlert {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<LevelAlert, Box<dyn Error + Sync + Send>> {
        match std::str::from_utf8(raw)? {
            "One" => Ok(LevelAlert::One),
            "Two" => Ok(LevelAlert::Two),
            "Three" => Ok(LevelAlert::Three),
            other => Err(format!("Unknown variant: {}", other).into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "level_alert"
    }
}

impl ToSql for LevelAlert {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut bytes::BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let value = match *self {
            LevelAlert::One => "One",
            LevelAlert::Two => "Two",
            LevelAlert::Three => "Three",
        };
        out.extend_from_slice(value.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "level_alert"
    }

    to_sql_checked!();
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
/// Alert struct
pub struct Alert {
    pub id: i32,
    pub user_id: i32,
    pub created_at: GraphQLDate,
    pub area: String,
    pub level: LevelAlert,
    pub reached_users: i32,
}

#[derive(InputObject)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(InputObject)]
/// Alert input struct
pub struct AlertInput {
    pub points: Vec<Point>,
    pub level: LevelAlert,
}

/// Get alerts from the database
pub async fn get_alerts<'ctx>(
    ctx: &Context<'ctx>,

    // Optional limit results
    limit: Option<i64>,

    // Optional offset results. It should be used with limit field.
    offset: Option<i64>,
) -> Result<Option<Vec<Alert>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(_) => {
            let rows = client
                .query(
                    "SELECT id, user_id, created_at, ST_AsText(area) as area, level, reached_users
                    FROM alerts
                    ORDER BY id DESC
                    LIMIT $1
                    OFFSET $2",
                    &[&limit.unwrap_or(20), &offset.unwrap_or(0)],
                )
                .await
                .unwrap();

            let positions: Vec<Alert> = rows
                .iter()
                .map(|row| Alert {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created_at: GraphQLDate(Utc::now()),
                    area: row.get("area"),
                    level: row.get("level"),
                    reached_users: row.get("reached_users"),
                })
                .collect();

            Ok(Some(positions))
        }
    }
}
