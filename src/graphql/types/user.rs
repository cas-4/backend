use crate::{errors::AppError, state::AppState};
use async_graphql::{Context, Object};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

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

pub async fn get_users<'ctx>(
    ctx: &Context<'ctx>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Option<Vec<User>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(_claims) => {
            let rows = client
                .query(
                    "SELECT id, email, password, is_admin FROM users LIMIT $1 OFFSET $2",
                    &[&limit.unwrap_or(20), &offset.unwrap_or(0)],
                )
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

pub async fn find_user(client: &Client, id: i32) -> Result<User, AppError> {
    let rows = client
        .query(
            "SELECT id, email, password, is_admin FROM users WHERE id = $1",
            &[&id],
        )
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

    if users.len() == 1 {
        Ok(users[0].clone())
    } else {
        Err(AppError::NotFound("User".to_string()))
    }
}
