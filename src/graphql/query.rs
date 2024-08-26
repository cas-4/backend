use crate::graphql::types::*;
use async_graphql::{Context, Object};

/// Query struct
pub struct Query;

#[Object]
impl Query {
    /// Returns the API version. It is like a "greet" function
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

    /// Returns all the positions
    async fn positions<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Filter by user id")] user_id: Option<i32>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<position::Position>>, String> {
        position::get_positions(ctx, user_id, limit, offset).await
    }

    /// Returns all the last positions for each user.
    /// It is restricted to only admin users.
    async fn last_positions<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Filter by moving activity")] moving_activity: Option<
            position::MovingActivity,
        >,
    ) -> Result<Option<Vec<position::Position>>, String> {
        position::last_positions(ctx, moving_activity).await
    }

    /// Returns all the positions
    async fn alerts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<alert::Alert>>, String> {
        alert::get_alerts(ctx, limit, offset).await
    }
}
