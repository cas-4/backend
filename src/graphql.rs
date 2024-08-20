use std::sync::Arc;

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

// use crate::state::AppState;

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    /// Returns the sum of a and b
    async fn add<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
        #[graphql(desc = "First value")] a: i32,
        #[graphql(desc = "Second value")] b: Option<i32>,
    ) -> i32 {
        // let state = ctx.data::<AppState>().unwrap();
        // let client = &*state.client;
        //
        // // Perform a database query
        // let rows = client
        //     .query("SELECT owner FROM payment", &[])
        //     .await
        //     .unwrap();
        // for row in rows {
        //     let owner: String = row.get(0);
        //     println!("{owner}");
        // }

        match b {
            Some(x) => a + x,
            None => a,
        }
    }
}

pub async fn graphql_handler(
    schema: Arc<Schema<Query, EmptyMutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
