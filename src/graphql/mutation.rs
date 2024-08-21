use crate::graphql::types::jwt;
use crate::state::AppState;
use async_graphql::{Context, Error, FieldResult, Object};

pub struct Mutation;

#[Object]
impl Mutation {
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
            let claims = jwt::Claims::new(id[0]);
            let token = claims.get_token().unwrap();
            Ok(jwt::AuthBody::new(token))
        } else {
            Err(Error::new("Invalid email or password"))
        }
    }
}
