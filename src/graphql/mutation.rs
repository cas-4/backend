use crate::{
    dates::GraphQLDate,
    graphql::types::{
        jwt::{self, Authentication},
        position,
    },
    state::AppState,
};
use async_graphql::{Context, Error, FieldResult, Object};
use chrono::Utc;

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
            Ok(jwt::AuthBody::new(token))
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
                        RETURNING id, user_id, created_at, ST_Y(location::geometry) AS latitude, ST_X(location::geometry) AS longitude, activity
                        ",
                        &[
                            &claims.user_id,
                            &input.latitude,
                            &input.longitude,
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
                        created_at: GraphQLDate(Utc::now()),
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
