use crate::graphql::types::user;
use async_graphql::{Context, Object};

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    /// Returns all the users
    async fn users<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<Vec<user::User>>, String> {
        user::get_users(ctx).await
    }
}
