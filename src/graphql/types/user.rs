use crate::state::AppState;
use async_graphql::{Context, Object};
use serde::{Deserialize, Serialize};

use super::jwt::Authentication;

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

pub async fn get_users<'ctx>(ctx: &Context<'ctx>) -> Result<Option<Vec<User>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(_claims) => {
            let rows = client
                .query("SELECT id, email, password, is_admin FROM users", &[])
                .await
                .unwrap();

            let users: Vec<User> = rows
                .iter()
                .map(|row| User {
                    id: row.get("id"),
                    email: row.get("email"),
                    password: row.get("password"),
                    is_admin: row.get("is_admin"),
                })
                .collect();

            Ok(Some(users))
        }
    }
}
