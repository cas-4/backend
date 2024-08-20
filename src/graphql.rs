use async_graphql::{
    http::GraphiQLSource, Context, EmptyMutation, EmptySubscription, Object, Schema,
};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};

struct Query;

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
        match b {
            Some(x) => a + x,
            None => a,
        }
    }
}

pub async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

pub fn create_route() -> Router {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)))
}
