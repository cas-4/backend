use crate::graphql::types::{
    alert,
    jwt::{self},
    position,
    user::{self},
};
use async_graphql::{Context, FieldResult, Object};

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
        jwt::mutations::login(ctx, input).await
    }

    /// Make GraphQL call to register a notification device token for the user.
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -H "Authorization: Bearer ***" \
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
        user::mutations::register_device(ctx, input).await
    }

    /// Make GraphQL call to edit their passowrd.
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -H "Authorization: Bearer ***" \
    /// -d '{
    ///   "query": "mutation UserPasswordEdit($input: UserPasswordEdit!) { userPasswordEdit(input: $input) { id email name address is_admin } }",
    ///   "variables": {
    ///     "input": {
    ///       "password1": "***",
    ///       "password2": "***"
    ///     }
    ///   }
    /// }'
    /// ```
    async fn user_password_edit<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: user::UserPasswordEdit,
    ) -> FieldResult<user::User> {
        user::mutations::user_password_edit(ctx, input).await
    }

    /// Make GraphQL call to edit an user. Not admins can edit only the user linked to the access
    /// token used.
    ///
    /// Example:
    /// ```text
    /// curl -X POST http://localhost:8000/graphql \
    /// -H "Content-Type: application/json" \
    /// -H "Authorization: Bearer ***" \
    /// -d '{
    ///   "query": "mutation UserEdit($input: UserEdit!, $id: Int!) { userEdit(input: $input, id: $id) { id email name address is_admin } }",
    ///   "variables": {
    ///     "input": {
    ///       "email": "mario.rossi@example.com",
    ///       "name": "Mario Rossi",
    ///       "address": ""
    ///     },
    ///     "id": 42
    ///   }
    /// }'
    /// ```
    async fn user_edit<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: user::UserEdit,
        id: i32,
    ) -> FieldResult<user::User> {
        user::mutations::user_edit(ctx, input, id).await
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
        position::mutations::new_position(ctx, input).await
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
        alert::mutations::new_alert(ctx, input).await
    }
}
