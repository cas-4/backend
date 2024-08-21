use crate::{graphql::types::User, state::AppState};
use async_graphql::{Context, Object};

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    /// Returns all the users
    async fn users<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<Vec<User>>, String> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let rows = client
            .query("SELECT id, email, password, is_admin FROM users", &[])
            .await
            .unwrap();

        let users: Vec<User> = rows
            .iter()
            .map(|row| User {
                id: row.get("id"),
                email: row.get("email"),
                password: row.get("password"),
                is_admin: row.get("is_admin"),
            })
            .collect();

        Ok(Some(users))
    }
}
