use crate::graphql::mutation::Mutation;
use crate::graphql::query::Query;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;

use super::types::jwt::Authentication;

pub async fn graphql_handler(
    schema: Extension<Schema<Query, Mutation, EmptySubscription>>,
    auth: Authentication,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.0.data(auth)).await.into()
}
