use crate::{
    errors::AppError,
    expo,
    graphql::types::{
        jwt::Authentication,
        notification::{LevelAlert, Notification},
        user::find_user,
    },
    state::AppState,
};
use async_graphql::{Context, FieldResult, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    pub area_level2: String,
    pub area_level3: String,
    pub text1: String,
    pub text2: String,
    pub text3: String,
    pub audio1: Vec<u8>,
    pub audio2: Vec<u8>,
    pub audio3: Vec<u8>,
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
    pub text1: String,
    pub text2: String,
    pub text3: String,
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
    ) -> Result<Option<Vec<Alert>>, AppError> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;
        let auth: &Authentication = ctx.data()?;
        match auth {
            Authentication::NotLogged => Err(AppError::Unauthorized),
            Authentication::Logged(_) => {
                let rows = match id {
                    Some(id) => {
                        client
                            .query(
                                "SELECT id,
                                    user_id,
                                    extract(epoch from created_at)::double precision as created_at,
                                    ST_AsText(area) as area,
                                    ST_AsText(ST_Buffer(area::geography, 1000)) as area_level2,
                                    ST_AsText(ST_Buffer(area::geography, 2000)) as area_level3,
                                    text1,
                                    text2,
                                    text3,
                                    audio1,
                                    audio2,
                                    audio3,
                                    reached_users
                                FROM alerts
                                WHERE id = $1",
                                &[&id],
                            )
                            .await?
                    }
                    None => {
                        client
                            .query(
                                "SELECT id,
                                    user_id,
                                    extract(epoch from created_at)::double precision as created_at,
                                    ST_AsText(area) as area,
                                    ST_AsText(ST_Buffer(area::geography, 1000)) as area_level2,
                                    ST_AsText(ST_Buffer(area::geography, 2000)) as area_level3,
                                    text1,
                                    text2,
                                    text3,
                                    audio1,
                                    audio2,
                                    audio3,
                                    reached_users
                                FROM alerts
                                ORDER BY id DESC
                                LIMIT $1
                                OFFSET $2",
                                &[&limit.unwrap_or(20), &offset.unwrap_or(0)],
                            )
                            .await?
                    }
                };

                let alerts: Vec<Alert> = rows
                    .iter()
                    .map(|row| Alert {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        area: row.get("area"),
                        area_level2: row.get("area_level2"),
                        area_level3: row.get("area_level3"),
                        text1: row.get("text1"),
                        text2: row.get("text2"),
                        text3: row.get("text3"),
                        audio1: row.get("audio1"),
                        audio2: row.get("audio2"),
                        audio3: row.get("audio3"),
                        reached_users: row.get("reached_users"),
                    })
                    .collect();

                Ok(Some(alerts))
            }
        }
    }
}

pub mod mutations {
    use crate::{audio, graphql::types::position::Position};

    use super::*;

    /// Create a new alert
    pub async fn new_alert<'ctx>(ctx: &Context<'ctx>, input: AlertInput) -> FieldResult<Alert> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data()?;
        match auth {
            Authentication::NotLogged => Err(AppError::NotFound("Owner".to_string()).into()),
            Authentication::Logged(claims) => {
                let claim_user = find_user(client, claims.user_id).await?;
                if !claim_user.is_admin {
                    return Err(AppError::Unauthorized.into());
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
                let rows = match client.query(&valid_query, &[]).await {
                    Ok(r) => r,
                    Err(_) => {
                        return Err(AppError::BadRequest("Polygon is not valid".to_string()).into());
                    }
                };

                let is_valid: bool = rows[0].get("is_valid");
                if !is_valid {
                    return Err(AppError::BadRequest("Polygon is not valid".to_string()).into());
                }

                // This is like a cache system to avoid multiple alerts for the same area within 10
                // minutes of interval
                if let Some(previous_alert) = client.query(
                    &format!(
                        "SELECT
                            id, user_id, extract(epoch from created_at)::double precision as created_at,
                            ST_AsText(area) as area,
                            ST_AsText(ST_Buffer(area::geography, 1000)) as area_level2,
                            ST_AsText(ST_Buffer(area::geography, 2000)) as area_level3,
                            text1, text2, text3,
                            audio1, audio2, audio3,
                            reached_users
                        FROM alerts WHERE area = {} AND created_at >= NOW() - INTERVAL '10 MINUTE'",
                        polygon
                    ),
                    &[]
                ).await?
                    .iter()
                    .map(|row| Alert {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        area: row.get("area"),
                        area_level2: row.get("area_level2"),
                        area_level3: row.get("area_level3"),
                        text1: row.get("text1"),
                        text2: row.get("text2"),
                        text3: row.get("text3"),
                        audio1: row.get("audio1"),
                        audio2: row.get("audio2"),
                        audio3: row.get("audio3"),
                        reached_users: row.get("reached_users"),
                    })
                    .collect::<Vec<Alert>>()
                    .first()
                    .cloned() {
                        return Ok(previous_alert);
                }

                let audio1 = match audio::tts(&input.text1).await {
                    Ok(content) => content,
                    Err(e) => {
                        tracing::error!("Error for `{}`: {}", &input.text1, e);
                        bytes::Bytes::new()
                    }
                };

                let audio2 = match audio::tts(&input.text2).await {
                    Ok(content) => content,
                    Err(e) => {
                        tracing::error!("Error for `{}`: {}", &input.text2, e);
                        bytes::Bytes::new()
                    }
                };

                let audio3 = match audio::tts(&input.text3).await {
                    Ok(content) => content,
                    Err(e) => {
                        tracing::error!("Error for `{}`: {}", &input.text3, e);
                        bytes::Bytes::new()
                    }
                };

                let insert_query = format!(
                    "INSERT INTO alerts (user_id, area, text1, text2, text3, audio1, audio2, audio3)
                    VALUES($1, {}, $2, $3, $4, $5, $6, $7)
                    RETURNING
                    id, user_id, extract(epoch from created_at)::double precision as created_at,
                    ST_AsText(area) as area,
                    ST_AsText(ST_Buffer(area::geography, 1000)) as area_level2,
                    ST_AsText(ST_Buffer(area::geography, 2000)) as area_level3,
                    text1, text2, text3,
                    audio1, audio2, audio3,
                    reached_users",
                    polygon
                );

                let rows = client
                    .query(
                        &insert_query,
                        &[
                            &claims.user_id,
                            &input.text1,
                            &input.text2,
                            &input.text3,
                            &audio1.to_vec(),
                            &audio2.to_vec(),
                            &audio3.to_vec(),
                        ],
                    )
                    .await?;
                let mut alert = rows
                    .iter()
                    .map(|row| Alert {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        area: row.get("area"),
                        area_level2: row.get("area_level2"),
                        area_level3: row.get("area_level3"),
                        text1: row.get("text1"),
                        text2: row.get("text2"),
                        text3: row.get("text3"),
                        audio1: row.get("audio1"),
                        audio2: row.get("audio2"),
                        audio3: row.get("audio3"),
                        reached_users: row.get("reached_users"),
                    })
                    .collect::<Vec<Alert>>()
                    .first()
                    .cloned()
                    .ok_or_else(|| AppError::BadRequest("Failed to create alert".to_string()))?;

                struct Level<'a> {
                    text: &'a str,
                    distance: f64,
                }

                let levels = vec![
                    Level {
                        text: "One",
                        distance: 0f64,
                    },
                    Level {
                        text: "Two",
                        distance: 1000f64,
                    },
                    Level {
                        text: "Three",
                        distance: 2000f64,
                    },
                ];

                let mut alerted_positions: Vec<i32> = vec![];

                // Send notifications for each available level
                for level in levels {
                    let positions: Vec<Position> = client
                        .query(
                            "SELECT id, user_id, extract(epoch from created_at)::double precision as created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                            FROM positions p
                            WHERE ST_DWithin(
                                    p.location::geography,
                                    (SELECT area::geography FROM alerts WHERE id = $1),
                                    $2
                                )
                            AND id = (
                                SELECT MAX(id)
                                FROM positions
                                WHERE user_id = p.user_id
                            )",
                            &[&alert.id, &level.distance],
                        )
                        .await?
                        .iter()
                        .map(|row| Position {
                            id: row.get("id"),
                            user_id: row.get("user_id"),
                            created_at: row.get::<_, f64>("created_at") as i64,
                            latitude: row.get("latitude"),
                            longitude: row.get("longitude"),
                            moving_activity: row.get("activity"),
                        })
                        .filter(|p| !alerted_positions.contains(&p.id))
                        .collect();

                    let mut notification_ids = vec![];
                    for p in &positions {
                        let notification = Notification::insert_db(
                            client,
                            alert.id,
                            p,
                            LevelAlert::from_str(level.text).unwrap(),
                        )
                        .await?;
                        notification_ids.push(notification);
                    }

                    alert.reached_users += notification_ids.len() as i32;
                    // Users placeholders
                    let placeholders: Vec<String> =
                        positions.iter().map(|p| format!("{}", p.user_id)).collect();

                    if !placeholders.is_empty() {
                        let query = format!(
                            "SELECT DISTINCT u.notification_token FROM users u
                            WHERE u.id IN ({}) AND notification_token IS NOT NULL",
                            placeholders.join(", ")
                        );

                        let tokens: Vec<String> = client
                            .query(&query, &[])
                            .await?
                            .iter()
                            .map(|row| {
                                format!("ExponentPushToken[{}]", row.get::<usize, String>(0))
                            })
                            .collect();

                        if tokens.len() > 0 {
                            expo::send(
                                (*state.expo).clone(),
                                tokens,
                                "New Alert!".to_string(),
                                match level.text {
                                    "One" => alert.text1.clone(),
                                    "Two" => alert.text2.clone(),
                                    "Three" => alert.text3.clone(),
                                    _ => "Check it out in app!".to_string(),
                                },
                            )
                            .await?;
                        }
                    }

                    alerted_positions.extend(positions.iter().map(|p| p.id).collect::<Vec<i32>>());
                }

                client
                    .query(
                        "UPDATE alerts SET reached_users = $1 WHERE id = $2",
                        &[&alert.reached_users, &alert.id],
                    )
                    .await?;

                Ok(alert)
            }
        }
    }
}
