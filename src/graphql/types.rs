use async_graphql::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
}

#[Object]
impl User {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn email(&self) -> String {
        self.email.clone()
    }

    async fn password(&self) -> String {
        String::from("******")
    }

    async fn is_admin(&self) -> bool {
        self.is_admin
    }
}
