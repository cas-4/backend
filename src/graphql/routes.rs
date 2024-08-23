use crate::graphql::mutation::Mutation;
use crate::graphql::query::Query;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;

use super::types::jwt::Authentication;

/// Handler for GraphQL route.
/// It executs the schema using the authorization as appdata
pub async fn graphql_handler(
    schema: Extension<Schema<Query, Mutation, EmptySubscription>>,
    auth: Authentication,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.0.data(auth)).await.into()
}
