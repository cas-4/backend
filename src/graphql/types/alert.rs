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
                        reached_users: row.get("reached_users"),
                    })
                    .collect();

                Ok(Some(alerts))
            }
        }
    }
}

pub mod mutations {
    use crate::audio;

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

                let insert_query = format!(
                    "INSERT INTO alerts (user_id, area, text1, text2, text3)
                    VALUES($1, {}, $2, $3, $4)
                    RETURNING
                    id, user_id, extract(epoch from created_at)::double precision as created_at,
                    ST_AsText(area) as area,
                    ST_AsText(ST_Buffer(area::geography, 1000)) as area_level2,
                    ST_AsText(ST_Buffer(area::geography, 2000)) as area_level3,
                    text1, text2, text3,
                    reached_users",
                    polygon
                );

                let rows = client
                    .query(
                        &insert_query,
                        &[&claims.user_id, &input.text1, &input.text2, &input.text3],
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

                let mut positions: Vec<i32> = vec![];

                // Send notifications for each available level
                for level in levels {
                    let position_ids: Vec<i32> = client
                        .query(
                            "SELECT id
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
                        .map(|row| row.get(0))
                        .filter(|id| !positions.contains(id))
                        .collect();

                    let mut notification_ids = vec![];
                    for id in &position_ids {
                        let notification = Notification::insert_db(
                            client,
                            alert.id,
                            *id,
                            LevelAlert::from_str(level.text).unwrap(),
                        )
                        .await?;
                        notification_ids.push(notification);
                    }

                    alert.reached_users += notification_ids.len() as i32;
                    let placeholders: Vec<String> = (1..=position_ids.len())
                        .map(|i| format!("${}", i))
                        .collect();

                    if !placeholders.is_empty() {
                        let query = format!(
                            "SELECT DISTINCT u.notification_token FROM positions p JOIN users u ON u.id = p.user_id
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
                            .await?
                            .iter()
                            .map(|row| {
                                format!("ExponentPushToken[{}]", row.get::<usize, String>(0))
                            })
                            .collect();

                        expo::send(
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

                    positions.extend(position_ids);
                }

                client
                    .query(
                        "UPDATE alerts SET reached_users = $1 WHERE id = $2",
                        &[&alert.reached_users, &alert.id],
                    )
                    .await?;

                if let Err(e) = audio::tts(
                    alert.text1.clone(),
                    format!("alert-{}-text-1.mp3", alert.id),
                )
                .await
                {
                    tracing::error!(
                        "Error for `{}`: {}",
                        format!("alert-{}-text-1.mp3", alert.id),
                        e
                    );
                }

                if let Err(e) = audio::tts(
                    alert.text2.clone(),
                    format!("alert-{}-text-2.mp3", alert.id),
                )
                .await
                {
                    tracing::error!(
                        "Error for `{}`: {}",
                        format!("alert-{}-text-2.mp3", alert.id),
                        e
                    );
                }
                if let Err(e) = audio::tts(
                    alert.text3.clone(),
                    format!("alert-{}-text-3.mp3", alert.id),
                )
                .await
                {
                    tracing::error!(
                        "Error for `{}`: {}",
                        format!("alert-{}-text-3.mp3", alert.id),
                        e
                    );
                }

                Ok(alert)
            }
        }
    }
}
