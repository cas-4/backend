use crate::{
    expo,
    graphql::types::{jwt::Authentication, notification::Notification, user::find_user},
    state::AppState,
};
use async_graphql::{Context, Enum, FieldResult, InputObject, SimpleObject};
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

#[derive(Serialize, Deserialize)]
pub struct PolygonValid {
    pub is_valid: bool,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
/// Alert struct
pub struct Alert {
    pub id: i32,
    pub user_id: i32,
    pub created_at: i64,
    pub area: String,
    pub extended_area: String,
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

pub mod query {
    use super::*;

    /// Get alerts from the database
    pub async fn get_alerts<'ctx>(
        ctx: &Context<'ctx>,

        // Optional filter by id.
        id: Option<i32>,

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
                let rows = match id {
                    Some(id) => client
                        .query(
                            "SELECT id,
                            user_id,
                            extract(epoch from created_at)::double precision as created_at,
                            ST_AsText(area) as area,
                            ST_AsText(
                                ST_Buffer(
                                    area::geography,
                                    CASE
                                        WHEN level = 'One' THEN 0
                                        WHEN level = 'Two' THEN 1000
                                        WHEN level = 'Three' THEN 2000
                                        ELSE 0
                                    END
                                )
                            ) as extended_area,
                            level,
                            reached_users
                    FROM alerts
                    WHERE id = $1",
                            &[&id],
                        )
                        .await
                        .unwrap(),
                    None => client
                        .query(
                            "SELECT id,
                        user_id,
                        extract(epoch from created_at)::double precision as created_at,
                        ST_AsText(area) as area,
                        ST_AsText(
                            ST_Buffer(
                                area::geography,
                                CASE
                                    WHEN level = 'One' THEN 0
                                    WHEN level = 'Two' THEN 1000
                                    WHEN level = 'Three' THEN 2000
                                    ELSE 0
                                END
                            )
                        ) as extended_area,
                        level,
                        reached_users
                    FROM alerts
                    ORDER BY id DESC
                    LIMIT $1
                    OFFSET $2",
                            &[&limit.unwrap_or(20), &offset.unwrap_or(0)],
                        )
                        .await
                        .unwrap(),
                };

                let alerts: Vec<Alert> = rows
                    .iter()
                    .map(|row| Alert {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        area: row.get("area"),
                        extended_area: row.get("extended_area"),
                        level: row.get("level"),
                        reached_users: row.get("reached_users"),
                    })
                    .collect();

                Ok(Some(alerts))
            }
        }
    }
}

pub mod mutations {
    use super::*;

    /// Create a new alert
    pub async fn new_alert<'ctx>(ctx: &Context<'ctx>, input: AlertInput) -> FieldResult<Alert> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err(async_graphql::Error::new("Can't find the owner")),
            Authentication::Logged(claims) => {
                let claim_user = find_user(client, claims.user_id).await.unwrap();
                if !claim_user.is_admin {
                    return Err(async_graphql::Error::new("Unauthorized"));
                }

                let points: String = input
                    .points
                    .iter()
                    .map(|x| {
                        format!(
                            "ST_SetSRID(ST_MakePoint({}, {}), 4326)",
                            x.longitude, x.latitude
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(",");

                let polygon = format!("ST_MakePolygon(ST_MakeLine(ARRAY[{}]))", points);

                let valid_query = format!("SELECT ST_IsValid({}) as is_valid", polygon);
                let rows = client.query(&valid_query, &[]).await.unwrap();

                let is_valid: bool = rows[0].get("is_valid");
                if !is_valid {
                    return Err(async_graphql::Error::new("Polygon is not valid"));
                }

                let insert_query = format!(
                    "INSERT INTO alerts (user_id, area, level)
                    VALUES($1, {}, $2)
                    RETURNING id, user_id, extract(epoch from created_at)::double precision as created_at, ST_AsText(area) as area,
                    ST_AsText(ST_Buffer(area::geography, CASE WHEN level = 'One' THEN 0 WHEN level = 'Two' THEN 1000 WHEN level = 'Three' THEN 2000 ELSE 0 END)) as extended_area, level, reached_users",
                    polygon
                );

                let rows = client
                    .query(&insert_query, &[&claims.user_id, &input.level])
                    .await
                    .unwrap();
                let mut alert = rows
                    .iter()
                    .map(|row| Alert {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        area: row.get("area"),
                        extended_area: row.get("extended_area"),
                        level: row.get("level"),
                        reached_users: row.get("reached_users"),
                    })
                    .collect::<Vec<Alert>>()
                    .first()
                    .cloned()
                    .ok_or_else(|| async_graphql::Error::new("Failed to create alert"))?;

                let distance: f64 = match alert.level {
                    LevelAlert::One => 0.0,
                    LevelAlert::Two => 1000.0,
                    LevelAlert::Three => 2000.0,
                };

                let position_ids: Vec<i32> = client
                    .query(
                        "
                        SELECT id FROM positions
                        WHERE ST_DWithin(
                            location::geography,
                            (SELECT area::geography FROM alerts WHERE id = $1),
                            $2
                        )",
                        &[&alert.id, &distance],
                    )
                    .await
                    .unwrap()
                    .iter()
                    .map(|row| row.get(0))
                    .collect();

                let mut notification_ids = vec![];
                for id in &position_ids {
                    let notification = Notification::insert_db(client, alert.id, *id)
                        .await
                        .unwrap();
                    notification_ids.push(notification);
                }

                alert.reached_users = notification_ids.len() as i32;
                client
                    .query(
                        "UPDATE alerts SET reached_users = $1 WHERE id = $2",
                        &[&alert.reached_users, &alert.id],
                    )
                    .await
                    .unwrap();

                let placeholders: Vec<String> = (1..=position_ids.len())
                    .map(|i| format!("${}", i))
                    .collect();
                let query = format!(
                    "SELECT u.notification_token FROM positions p JOIN users u ON u.id = p.user_id
                    WHERE p.id IN ({}) AND notification_token IS NOT NULL",
                    placeholders.join(", ")
                );

                let tokens: Vec<String> = client
                    .query(
                        &query,
                        &position_ids
                            .iter()
                            .map(|id| id as &(dyn tokio_postgres::types::ToSql + Sync))
                            .collect::<Vec<&(dyn tokio_postgres::types::ToSql + Sync)>>(),
                    )
                    .await
                    .unwrap()
                    .iter()
                    .map(|row| format!("ExponentPushToken[{}]", row.get::<usize, String>(0)))
                    .collect();

                expo::send(
                    tokens,
                    "New Alert!".to_string(),
                    "Keep an eye open".to_string(),
                )
                .await
                .unwrap();

                Ok(alert)
            }
        }
    }
}
