use crate::{errors::AppError, state::AppState};
use async_graphql::{Context, Object};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use super::jwt::Authentication;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// User struct
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub address: Option<String>,
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

    async fn name(&self) -> String {
        self.name.clone().unwrap_or(String::default())
    }

    async fn address(&self) -> String {
        self.address.clone().unwrap_or(String::default())
    }

    async fn is_admin(&self) -> bool {
        self.is_admin
    }
}

/// Get users from the database
pub async fn get_users<'ctx>(
    ctx: &Context<'ctx>,

    // Optional limit results
    limit: Option<i64>,
    // Optional offset results. It should be used with limit field.
    offset: Option<i64>,
) -> Result<Option<Vec<User>>, String> {
    let state = ctx.data::<AppState>().expect("Can't connect to db");
    let client = &*state.client;
    let auth: &Authentication = ctx.data().unwrap();
    match auth {
        Authentication::NotLogged => Err("Unauthorized".to_string()),
        Authentication::Logged(claims) => {
            let claim_user = find_user(client, claims.user_id)
                .await
                .expect("Should not be here");

            if !claim_user.is_admin {
                return Err("Unauthorized".to_string());
            }

            let rows = client
                .query(
                    "SELECT id, email, password, name, address, is_admin FROM users LIMIT $1 OFFSET $2",
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
                    name: row.get("name"),
                    address: row.get("address"),
                    is_admin: row.get("is_admin"),
                })
                .collect();

            Ok(Some(users))
        }
    }
}

/// Find an user with id = `id` using the PostgreSQL `client`
pub async fn find_user(client: &Client, id: i32) -> Result<User, AppError> {
    let rows = client
        .query(
            "SELECT id, email, password, name, address, is_admin FROM users WHERE id = $1",
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
            name: row.get("name"),
            address: row.get("address"),
            is_admin: row.get("is_admin"),
        })
        .collect();

    if users.len() == 1 {
        Ok(users[0].clone())
    } else {
        Err(AppError::NotFound("User".to_string()))
    }
}
