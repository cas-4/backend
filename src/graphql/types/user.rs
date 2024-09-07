use crate::{errors::AppError, state::AppState};
use async_graphql::{Context, Error, FieldResult, InputObject, Object};
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
    pub notification_token: Option<String>,
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
        self.name.clone().unwrap_or_default()
    }

    async fn address(&self) -> String {
        self.address.clone().unwrap_or_default()
    }

    async fn notification_token(&self) -> String {
        String::from("******")
    }

    async fn is_admin(&self) -> bool {
        self.is_admin
    }
}

#[derive(InputObject, Debug)]
pub struct RegisterNotificationToken {
    pub token: String,
}

/// Find an user with id = `id` using the PostgreSQL `client`
pub async fn find_user(client: &Client, id: i32) -> Result<User, AppError> {
    let rows = client
        .query(
            "SELECT id, email, name, address, is_admin FROM users WHERE id = $1",
            &[&id],
        )
        .await
        .unwrap();

    let users: Vec<User> = rows
        .iter()
        .map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            password: String::new(),
            name: row.get("name"),
            address: row.get("address"),
            notification_token: None,
            is_admin: row.get("is_admin"),
        })
        .collect();

    if users.len() == 1 {
        Ok(users[0].clone())
    } else {
        Err(AppError::NotFound("User".to_string()))
    }
}

pub mod query {
    use super::*;

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
                        "SELECT id, email, name, address, is_admin FROM users LIMIT $1 OFFSET $2",
                        &[&limit.unwrap_or(20), &offset.unwrap_or(0)],
                    )
                    .await
                    .unwrap();

                let users: Vec<User> = rows
                    .iter()
                    .map(|row| User {
                        id: row.get("id"),
                        email: row.get("email"),
                        password: String::new(),
                        name: row.get("name"),
                        address: row.get("address"),
                        notification_token: None,
                        is_admin: row.get("is_admin"),
                    })
                    .collect();

                Ok(Some(users))
            }
        }
    }

    /// Get users from the database
    pub async fn get_user_by_id<'ctx>(ctx: &Context<'ctx>, id: i32) -> Result<User, String> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;
        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err("Unauthorized".to_string()),
            Authentication::Logged(claims) => {
                let claim_user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                let rows;
                if claim_user.is_admin {
                    rows = client
                        .query(
                            "SELECT id, email, name, address, is_admin FROM users
                            WHERE id = $1",
                            &[&id],
                        )
                        .await
                        .unwrap();
                } else if claims.user_id != id {
                    return Err("Unauthorized".to_string());
                } else {
                    rows = client
                        .query(
                            "SELECT id, email, name, address, is_admin FROM users
                            WHERE id = $1",
                            &[&claims.user_id],
                        )
                        .await
                        .unwrap();
                }

                let users: Vec<User> = rows
                    .iter()
                    .map(|row| User {
                        id: row.get("id"),
                        email: row.get("email"),
                        password: String::new(),
                        name: row.get("name"),
                        address: row.get("address"),
                        notification_token: None,
                        is_admin: row.get("is_admin"),
                    })
                    .collect();

                if users.is_empty() {
                    return Err("Not found".to_string());
                }

                Ok(users[0].clone())
            }
        }
    }
}

pub mod mutations {
    use super::*;

    /// Register device mutation edits the `notification_token` value for a logged user
    pub async fn register_device<'ctx>(
        ctx: &Context<'ctx>,
        input: RegisterNotificationToken,
    ) -> FieldResult<User> {
        let state = ctx.data::<AppState>().expect("Can't connect to db");
        let client = &*state.client;

        let auth: &Authentication = ctx.data().unwrap();
        match auth {
            Authentication::NotLogged => Err(Error::new("Can't find the owner")),
            Authentication::Logged(claims) => {
                let user = find_user(client, claims.user_id)
                    .await
                    .expect("Should not be here");

                client
                    .query(
                        "UPDATE users SET notification_token = $1 WHERE id = $2",
                        &[&input.token, &claims.user_id],
                    )
                    .await
                    .unwrap();

                Ok(user)
            }
        }
    }
}
