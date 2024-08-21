use crate::graphql::mutation::Mutation;
use crate::graphql::query::Query;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use std::sync::Arc;

pub async fn graphql_handler(
    schema: Arc<Schema<Query, Mutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
