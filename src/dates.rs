use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A DateTime used as GraphQL type
pub struct GraphQLDate(pub DateTime<Utc>);

impl From<DateTime<Utc>> for GraphQLDate {
    fn from(dt: DateTime<Utc>) -> Self {
        GraphQLDate(dt)
    }
}

impl From<GraphQLDate> for DateTime<Utc> {
    fn from(my_dt: GraphQLDate) -> Self {
        my_dt.0
    }
}

#[Scalar]
impl ScalarType for GraphQLDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(s) = &value {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| GraphQLDate(dt.with_timezone(&Utc)))
                .map_err(|e| InputValueError::custom(format!("Invalid DateTime: {}", e)))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}
