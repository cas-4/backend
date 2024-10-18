use crate::{errors::AppError, graphql::types::*};
use async_graphql::{Context, Object};

/// Query struct
pub struct Query;

#[Object]
impl Query {
    /// Returns the API version. It is like a "greet" function
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    /// Returns all the users. It is restricted to admins only.
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{users(limit: 2) { id, email, password, name, address, isAdmin }}"}'
    /// ```
    async fn users<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<user::User>>, AppError> {
        user::query::get_users(ctx, limit, offset).await
    }

    /// Returns an user by ID. Admins can check everyone.
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{user(id: 1) { id, email, password, name, address, isAdmin }}"}'
    /// ```
    async fn user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "User to find")] id: i32,
    ) -> Result<user::User, AppError> {
        user::query::get_user_by_id(ctx, id).await
    }

    /// Returns all the positions. It is restricted to admins only.
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{positions(movingActivity: IN_VEHICLE) {id, userId, createdAt, latitude, longitude, movingActivity}}"}'
    /// ```
    async fn positions<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Filter by moving activity")] moving_activity: Option<
            Vec<position::MovingActivity>,
        >,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<position::Position>>, AppError> {
        position::query::get_positions(ctx, moving_activity, limit, offset).await
    }

    /// Returns all the positions
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{alerts(id: 12) {id, userId, createdAt, area, areaLevel2, areaLevel3, text1, text2, text3}}"}'
    /// ```
    async fn alerts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Filter by ID")] id: Option<i32>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<alert::Alert>>, AppError> {
        alert::query::get_alerts(ctx, id, limit, offset).await
    }

    /// Returns all the notifications. They can be filtered by an alert id.
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{notifications(seen: false alertId: 1) {
    /// id,
    /// alert { id, userId, createdAt, area, areaLevel2, areaLevel3, text1, text2, text3, reachedUsers },
    /// position {id, userId, createdAt, latitude, longitude, movingActivity},
    /// seen,
    /// level,
    /// createdAt
    /// }}"}'
    /// ```
    async fn notifications<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Show only seen or not notifications")] seen: Option<bool>,
        #[graphql(desc = "Filter by ID")] id: Option<i32>,
        #[graphql(desc = "Filter by alert ID")] alert_id: Option<i32>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<notification::Notification>>, AppError> {
        notification::query::get_notifications(ctx, seen, id, alert_id, limit, offset).await
    }
}
