use crate::{
    errors::AppError,
    graphql::types::{
        alert::Alert,
        jwt::Authentication,
        position::{MovingActivity, Position},
        user::find_user,
    },
    state::AppState,
};
use async_graphql::{Context, Enum, FieldResult, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use std::{error::Error, str::FromStr};
use tokio_postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use tokio_postgres::Client;

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

impl FromStr for LevelAlert {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "One" => Ok(LevelAlert::One),
            "Two" => Ok(LevelAlert::Two),
            "Three" => Ok(LevelAlert::Three),
            _ => Err(String::from("Can't parse this value as Level")),
        }
    }
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
/// Notification struct
pub struct Notification {
    pub id: i32,
    pub alert: Alert,
    pub user_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub moving_activity: MovingActivity,
    pub seen: bool,
    pub level: LevelAlert,
    pub created_at: i64,
}

#[derive(InputObject)]
/// Alert input struct
pub struct NotificationUpdateInput {
    pub id: i32,
    pub seen: bool,
}

impl Notification {
    /// Create a new notification into the database from an alert_id and a position.
    /// Returns the new ID.
    pub async fn insert_db(
        client: &Client,
        alert_id: i32,
        position: &Position,
        level: LevelAlert,
    ) -> Result<i32, AppError> {
        match client
            .query(
                "INSERT INTO notifications(alert_id, user_id, location, activity, level)
                VALUES($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326), $5, $6)
                RETURNING id
                ",
                &[
                    &alert_id,
                    &position.user_id,
                    &position.longitude,
                    &position.latitude,
                    &position.moving_activity,
                    &level,
                ],
            )
            .await
        {
            Ok(rows) => {
                let row = rows[0].clone();
                Ok(row.get("id"))
            }
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }
}

pub mod query {
    use super::*;

    /// Get notifications from the database
    pub async fn get_notifications<'ctx>(
        ctx: &Context<'ctx>,

        // Filter for `seen` field
        seen: Option<bool>,

        // Optional filter by id
        id: Option<i32>,

        // Optional filter by alert id
        alert_id: Option<i32>,

        // Optional limit results
        limit: Option<i64>,

        // Optional offset results. It should be used with limit field.
        offset: Option<i64>,
    ) -> Result<Option<Vec<Notification>>, AppError> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;
        let auth: &Authentication = ctx.data()?;
        match auth {
            Authentication::NotLogged => Err(AppError::Unauthorized),
            Authentication::Logged(claims) => {
                let claim_user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                let limit = limit.unwrap_or(20);
                let offset = offset.unwrap_or(0);

                let base_query = "SELECT n.id,
                                n.alert_id,
                                n.seen,
                                n.level,
                                extract(epoch from n.created_at)::double precision as created_at,
                                ST_Y(n.location::geometry) AS latitude,
                                ST_X(n.location::geometry) AS longitude,
                                n.activity,
                                n.user_id,
                                a.id as alert_id,
                                a.user_id as alert_user_id,
                                extract(epoch from a.created_at)::double precision as alert_created_at,
                                ST_AsText(a.area) as alert_area,
                                ST_AsText(ST_Buffer(a.area::geography, 1000)) as alert_area_level2,
                                ST_AsText(ST_Buffer(a.area::geography, 2000)) as alert_area_level3,
                                a.text1 as alert_text1,
                                a.text2 as alert_text2,
                                a.text3 as alert_text3,
                                a.audio1 as alert_audio1,
                                a.audio2 as alert_audio2,
                                a.audio3 as alert_audio3,
                                a.reached_users as alert_reached_users
                        FROM notifications n
                        JOIN alerts a ON n.alert_id = a.id".to_string();

                let base_query = match id {
                    Some(idn) => format!("{} WHERE n.id = {}", base_query, idn),
                    None => format!("{} WHERE 1=1", base_query),
                };

                let base_query = match seen {
                    Some(seen_status) if seen_status => format!("{} AND seen = 't'", base_query),
                    Some(_) => format!("{} AND seen = 'f'", base_query),
                    None => base_query,
                };

                let rows = match alert_id {
                    Some (ida) =>
                        client
                        .query(&format!(
                            "{base_query} AND n.user_id = $1 AND n.alert_id = $2 ORDER BY n.id DESC LIMIT $3 OFFSET $4",
                        ), &[&claim_user.id, &ida, &limit, &offset])
                        .await?,
                    None =>
                        client.query(
                            &format!("{base_query} AND n.user_id = $1 ORDER BY n.id DESC LIMIT $2 OFFSET $3"),
                            &[&claim_user.id, &limit, &offset],
                        )
                        .await?,
                };

                let notifications: Vec<Notification> = rows
                    .iter()
                    .map(|row| Notification {
                        id: row.get("id"),
                        alert: Alert {
                            id: row.get("alert_id"),
                            user_id: row.get("alert_user_id"),
                            created_at: row.get::<_, f64>("alert_created_at") as i64,
                            area: row.get("alert_area"),
                            area_level2: row.get("alert_area_level2"),
                            area_level3: row.get("alert_area_level3"),
                            text1: row.get("alert_text1"),
                            text2: row.get("alert_text2"),
                            text3: row.get("alert_text3"),
                            audio1: row.get("alert_audio1"),
                            audio2: row.get("alert_audio2"),
                            audio3: row.get("alert_audio3"),
                            reached_users: row.get("alert_reached_users"),
                        },
                        seen: row.get("seen"),
                        level: row.get("level"),
                        user_id: row.get("user_id"),
                        latitude: row.get("latitude"),
                        longitude: row.get("longitude"),
                        moving_activity: row.get("activity"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                    })
                    .collect();

                Ok(Some(notifications))
            }
        }
    }
}

pub mod mutations {
    use super::*;

    pub async fn notification_update<'ctx>(
        ctx: &Context<'ctx>,
        input: NotificationUpdateInput,
    ) -> FieldResult<Notification> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data()?;
        match auth {
            Authentication::NotLogged => Err(AppError::NotFound("Owner".to_string()).into()),
            Authentication::Logged(claims) => {
                let user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                let notification = client.query("SELECT n.id,
                                n.alert_id,
                                n.seen,
                                n.level,
                                extract(epoch from n.created_at)::double precision as created_at,
                                ST_Y(n.location::geometry) AS latitude,
                                ST_X(n.location::geometry) AS longitude,
                                n.activity,
                                n.user_id,
                                a.id as alert_id,
                                a.user_id as alert_user_id,
                                extract(epoch from a.created_at)::double precision as alert_created_at,
                                ST_AsText(a.area) as alert_area,
                                ST_AsText(ST_Buffer(a.area::geography, 1000)) as alert_area_level2,
                                ST_AsText(ST_Buffer(a.area::geography, 2000)) as alert_area_level3,
                                a.text1 as alert_text1,
                                a.text2 as alert_text2,
                                a.text3 as alert_text3,
                                a.audio1 as alert_audio1,
                                a.audio2 as alert_audio2,
                                a.audio3 as alert_audio3,
                                a.reached_users as alert_reached_users
                        FROM notifications n
                        JOIN alerts a ON n.alert_id = a.id
                        WHERE n.id = $1
                        ",
                       &[&input.id])
                    .await?
                    .iter()
                    .map(|row| Notification {
                        id: row.get("id"),
                        alert: Alert {
                            id: row.get("alert_id"),
                            user_id: row.get("alert_user_id"),
                            created_at: row.get::<_, f64>("alert_created_at") as i64,
                            area: row.get("alert_area"),
                            area_level2: row.get("alert_area_level2"),
                            area_level3: row.get("alert_area_level3"),
                            text1: row.get("alert_text1"),
                            text2: row.get("alert_text2"),
                            text3: row.get("alert_text3"),
                            audio1: row.get("alert_audio1"),
                            audio2: row.get("alert_audio2"),
                            audio3: row.get("alert_audio3"),
                            reached_users: row.get("alert_reached_users"),
                        },
                        seen: row.get("seen"),
                        level: row.get("level"),
                        user_id: row.get("user_id"),
                        latitude: row.get("latitude"),
                        longitude: row.get("longitude"),
                        moving_activity: row.get("activity"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                    })
                    .collect::<Vec<Notification>>()
                    .first()
                    .cloned()
                    .ok_or_else(|| AppError::NotFound("Notification".to_string()))?;

                if notification.user_id != user.id {
                    return Err(AppError::NotFound("Notification".to_string()).into());
                }

                client
                    .query(
                        "UPDATE notifications SET seen = $1 WHERE id = $2",
                        &[&input.seen, &input.id],
                    )
                    .await?;

                Ok(notification)
            }
        }
    }
}
