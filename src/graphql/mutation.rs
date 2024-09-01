use crate::{
    graphql::types::{
        alert,
        jwt::{self, Authentication},
        position,
        user::find_user,
    },
    state::AppState,
};
use async_graphql::{Context, Error, FieldResult, Object};

/// Mutation struct
pub struct Mutation;

#[Object]
impl Mutation {
    /// Make GraphQL login
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

    /// Make GraphQL request to create new position to track
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
                let claim_user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

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

                let polygon = format!(
                    "ST_MakePolygon(
                            ST_MakeLine(
                                ARRAY[{}]
                            )
                        )",
                    points
                );

                match client
                    .query(&format!("SELECT ST_IsValid({}) as is_valid", polygon), &[])
                    .await
                {
                    Ok(rows) => {
                        let valids: Vec<alert::PolygonValid> = rows
                            .iter()
                            .map(|row| alert::PolygonValid {
                                is_valid: row.get("is_valid"),
                            })
                            .collect();

                        if valids[0].is_valid == false {
                            return Err(Error::new("Polygon is not valid"));
                        }
                    }
                    Err(e) => return Err(e.into()),
                };

                let query = format!(
                    "INSERT INTO alerts (user_id, area, level)
                        VALUES($1, {}, $2)
                        RETURNING
                            id,
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
                            reached_users",
                    polygon
                );

                match client.query(&query, &[&claims.user_id, &input.level]).await {
                    Ok(rows) => {
                        let alerts: Vec<alert::Alert> = rows
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
                            .collect();

                        // TODO: Send notifications

                        Ok(alerts[0].clone())
                    }
                    Err(e) => Err(e.into()),
                }
            }
        }
    }
}
