use crate::{
    expo,
    graphql::types::{
        alert,
        jwt::{self, Authentication},
        notification, position,
        user::{self, find_user},
    },
    state::AppState,
};
use async_graphql::{Context, Error, FieldResult, Object};

/// Mutation struct
pub struct Mutation;

#[Object]
impl Mutation {
    /// Make GraphQL login
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -d '{
    ///   "query": "mutation Login($input: LoginCredentials!) { login(input: $input) { accessToken tokenType userId } }",
    ///   "variables": {
    ///     "input": {
    ///       "email": "***",
    ///       "password": "***"
    ///     }
    ///   }
    /// }'
    /// ```
    async fn login<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: jwt::LoginCredentials,
    ) -> FieldResult<jwt::AuthBody> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let password = sha256::digest(input.password);
        let rows = client
            .query(
                "SELECT id FROM users WHERE email = $1 AND password = $2",
                &[&input.email, &password],
            )
            .await
            .unwrap();

        let id: Vec<i32> = rows.iter().map(|row| row.get(0)).collect();
        if id.len() == 1 {
            // Create a new claim using the found ID
            let claims = jwt::Claims::new(id[0]);
            let token = claims.get_token().unwrap();
            Ok(jwt::AuthBody::new(token, id[0]))
        } else {
            Err(Error::new("Invalid email or password"))
        }
    }

    /// Make GraphQL call to register a notification device token for the user.
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -d '{
    ///   "query": "mutation RegisterDevice($input: RegisterNotificationToken!) { registerDevice(input: $input) { id name email } }",
    ///   "variables": {
    ///     "input": {
    ///       "token": "***",
    ///     }
    ///   }
    /// }'
    /// ```
    async fn register_device<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: user::RegisterNotificationToken,
    ) -> FieldResult<user::User> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err(Error::new("Can't find the owner")),
            Authentication::Logged(claims) => {
                let user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                client
                    .query(
                        "UPDATE users SET notification_token = $1 WHERE id = $2",
                        &[&input.token, &claims.user_id],
                    )
                    .await
                    .unwrap();

                Ok(user)
            }
        }
    }

    /// Make GraphQL request to create new position to track
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -H "Authorization: Bearer ***" \
    /// -d '{
    ///   "query": "mutation NewPosition($input: PositionInput!) { newPosition(input: $input) { id userId createdAt latitude longitude movingActivity } }",
    ///   "variables": {
    ///     "input": {
    ///       "latitude": 44.50800643571219,
    ///       "longitude": 11.299600981136905,
    ///       "movingActivity": "STILL"
    ///     }
    ///   }
    /// }'
    /// ```
    async fn new_position<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: position::PositionInput,
    ) -> FieldResult<position::Position> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err(Error::new("Can't find the owner")),
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

                let positions: Vec<position::Position> = rows
                    .iter()
                    .map(|row| position::Position {
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

    /// Make GraphQL request to create new alert. Only for admins.
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -H "Authorization: Bearer ****" \
    /// -d '{
    ///   "query": "mutation NewAlert($input: AlertInput!) { newAlert(input: $input) { id createdAt level } }",
    ///   "variables": {
    ///     "input": {
    ///       "points": [
    ///         { "latitude": 44.490025, "longitude": 11.311499},
    ///         { "latitude": 44.490361, "longitude": 11.327903},
    ///         { "latitude": 44.497280, "longitude": 11.327776},
    ///         { "latitude": 44.498321, "longitude": 11.312145},
    ///         { "latitude": 44.490025, "longitude": 11.311498}
    ///       ],
    ///       "level": "TWO"
    ///     }
    ///   }
    /// }
    async fn new_alert<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: alert::AlertInput,
    ) -> FieldResult<alert::Alert> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err(Error::new("Can't find the owner")),
            Authentication::Logged(claims) => {
                let claim_user = find_user(client, claims.user_id).await.unwrap();
                if !claim_user.is_admin {
                    return Err(Error::new("Unauthorized"));
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
                    return Err(Error::new("Polygon is not valid"));
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
                    .map(|row| alert::Alert {
                        id: row.get("id"),
                        user_id: row.get("user_id"),
                        created_at: row.get::<_, f64>("created_at") as i64,
                        area: row.get("area"),
                        extended_area: row.get("extended_area"),
                        level: row.get("level"),
                        reached_users: row.get("reached_users"),
                    })
                    .collect::<Vec<alert::Alert>>()
                    .first()
                    .cloned()
                    .ok_or_else(|| Error::new("Failed to create alert"))?;

                let distance: f64 = match alert.level {
                    alert::LevelAlert::One => 0.0,
                    alert::LevelAlert::Two => 1000.0,
                    alert::LevelAlert::Three => 2000.0,
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
                    let notification = notification::Notification::insert_db(client, alert.id, *id)
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
