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
    ) -> Result<Option<Vec<user::User>>, String> {
        user::get_users(ctx, limit, offset).await
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
    ) -> Result<user::User, String> {
        user::get_user_by_id(ctx, id).await
    }

    /// Returns all the positions
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{positions {id, userId, createdAt, latitude, longitude, movingActivity}}"}'
    /// ```
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
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"lastPositions(movingActivity: IN_VEHICLE) {id, userId, createdAt, latitude, longitude, movingActivity}}"}'
    /// ```
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
    ///
    /// Request example:
    /// ```text
    /// curl http://localhost:8000/graphql
    /// -H 'authorization: Bearer ***'
    /// -H 'content-type: application/json'
    /// -d '{"query":"{alerts(id: 12) {id, userId, createdAt, area, extendedArea, level}}"}'
    /// ```
    async fn alerts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Filter by ID")] id: Option<i32>,
        #[graphql(desc = "Limit results")] limit: Option<i64>,
        #[graphql(desc = "Offset results")] offset: Option<i64>,
    ) -> Result<Option<Vec<alert::Alert>>, String> {
        alert::get_alerts(ctx, id, limit, offset).await
    }
}
