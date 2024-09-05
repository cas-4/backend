use crate::{
    errors::AppError,
    graphql::types::{alert::Alert, jwt::Authentication, position::Position, user::find_user},
    state::AppState,
};
use async_graphql::{Context, SimpleObject};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
/// Notification struct
pub struct Notification {
    pub id: i32,
    pub alert: Alert,
    pub position: Position,
    pub seen: bool,
    pub created_at: i64,
}

impl Notification {
    /// Create a new notification into the database from an alert_id and a position_id.
    /// Returns the new ID.
    pub async fn insert_db(
        client: &Client,
        alert_id: i32,
        position_id: i32,
    ) -> Result<i32, AppError> {
        match client
            .query(
                "INSERT INTO notifications(alert_id, position_id)
                VALUES($1, $2)
                RETURNING id
                ",
                &[&alert_id, &position_id],
            )
            .await
        {
            Ok(rows) => {
                let row = rows[0].clone();
                Ok(row.get("id"))
            }
            Err(_) => Err(AppError::Database),
        }
    }
}

/// Get notifications from the database
pub async fn get_notifications<'ctx>(
    ctx: &Context<'ctx>,

    // Filter for `seen` field
    seen: bool,

    // Optional filter by alert id
    alert_id: Option<i32>,

    // Optional limit results
    limit: Option<i64>,

    // Optional offset results. It should be used with limit field.
    offset: Option<i64>,
) -> Result<Option<Vec<Notification>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(claims) => {
            let claim_user = find_user(client, claims.user_id)
                .await
                .expect("Should not be here");

            let limit = limit.unwrap_or(20);
            let offset = offset.unwrap_or(0);

            let base_query = "SELECT n.id,
                                n.alert_id,
                                n.position_id,
                                n.seen,
                                extract(epoch from n.created_at)::double precision as created_at,
                                a.id as alert_id,
                                a.user_id as alert_user_id,
                                extract(epoch from a.created_at)::double precision as alert_created_at,
                                ST_AsText(a.area) as alert_area,
                                ST_AsText(
                                    ST_Buffer(
                                        a.area::geography,
                                        CASE
                                            WHEN level = 'One' THEN 0
                                            WHEN level = 'Two' THEN 1000
                                            WHEN level = 'Three' THEN 2000
                                            ELSE 0
                                        END
                                    )
                                ) as alert_extended_area,
                                a.level as alert_level,
                                a.reached_users as alert_reached_users,
                                p.id as position_id,
                                p.user_id as position_user_id,
                                extract(epoch from p.created_at)::double precision as position_created_at,
                                ST_Y(p.location::geometry) AS position_latitude,
                                ST_X(p.location::geometry) AS position_longitude,
                                p.activity as position_activity
                        FROM notifications n
                        JOIN alerts a ON n.alert_id = a.id
                        JOIN positions p ON n.position_id = p.id".to_string();

            let rows = match alert_id {
                Some(id) if claim_user.is_admin =>
                        client
                        .query(&format!(
                            "{base_query} WHERE seen = $1 AND n.alert_id = $2 ORDER BY n.id DESC LIMIT $3 OFFSET $4",
                        ), &[&seen, &id, &limit, &offset])
                        .await
                        .unwrap(),
                Some (id) =>
                    client
                    .query(&format!(
                        "{base_query} WHERE seen = $1 AND p.user_id = $2 AND n.alert_id = $3 ORDER BY n.id DESC LIMIT $4 OFFSET $5",
                    ), &[&seen, &claim_user.id, &id, &limit, &offset])
                    .await
                    .unwrap(),
                None if claim_user.is_admin => client
                    .query(
                        &format!("{base_query} WHERE seen = $1 ORDER BY n.id DESC LIMIT $2 OFFSET $3"),
                        &[&seen, &limit, &offset],
                    )
                    .await
                    .unwrap(),
                None =>
                    client.query(
                        &format!("{base_query} WHERE seen = $1 AND p.user_id = $2 ORDER BY n.id DESC LIMIT $3 OFFSET $4"),
                        &[&seen, &claim_user.id, &limit, &offset],
                    )
                    .await
                    .unwrap(),
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
                        extended_area: row.get("alert_extended_area"),
                        level: row.get("alert_level"),
                        reached_users: row.get("alert_reached_users"),
                    },
                    position: Position {
                        id: row.get("position_id"),
                        user_id: row.get("position_user_id"),
                        created_at: row.get::<_, f64>("position_created_at") as i64,
                        latitude: row.get("position_latitude"),
                        longitude: row.get("position_longitude"),
                        moving_activity: row.get("position_activity"),
                    },
                    seen: row.get("seen"),
                    created_at: row.get::<_, f64>("created_at") as i64,
                })
                .collect();

            Ok(Some(notifications))
        }
    }
}
