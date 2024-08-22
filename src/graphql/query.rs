use crate::{errors::AppError, graphql::types::user};
use async_graphql::{Context, Object};

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    /// Returns all the users
    async fn users<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<user::User>>, String> {
        user::get_users(ctx, limit, offset).await
    }
}
