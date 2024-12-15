use crate::{config::DB_TABLE, prelude::*};
use serde::{Deserialize, Serialize};
use shared::user::UserId;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccount {
    pub id: UserId,
    pub user_token: String,
    pub created_at: String,
    pub roles: Vec<UserRole>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Copy)]
pub enum UserRole {
    Admin
}

impl UserAccount {
    pub async fn load(env: &Env, id: &UserId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE id = ?1",
                DB_TABLE.user_account
            ))
            .bind(&[id.into()])?
            .first::<UserAccount>(None)
            .await?
            .map(UserAccount::from)
            .ok_or(format!("Need to register (id {id})").into())
    }

    pub async fn _exists(env: &Env, id: &UserId) -> ApiResult<bool> {
        let res = get_d1(env)?
            .prepare(format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE omi_id = ?1)",
                DB_TABLE.user_account
            ))
            .bind(&[id.into()])?
            .raw::<u32>()
            .await?;

        let exists = res[0][0] == 1;

        Ok(exists)
    }

    pub async fn insert(env: &Env, id: &UserId, user_token: &str) -> ApiResult<()> {
        get_d1(env)?
            .prepare(format!(
                "INSERT INTO {} (id, user_token) VALUES (?1, ?2)",
                DB_TABLE.user_account
            ))
            .bind(&[id.into(), user_token.into()])?
            .run()
            .await?
            .into_result()
    }

    pub async fn update_user_token(env: &Env, id: &UserId, user_token: &str) -> ApiResult<()> {
        get_d1(env)?
            .prepare(format!(
                "UPDATE {} SET user_token = ?1 WHERE id = ?2",
                DB_TABLE.user_account
            ))
            .bind(&[user_token.into(), id.into()])?
            .run()
            .await?
            .into_result()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramAccount {
    pub id: i64,
    pub user_id: UserId,
    pub first_name: String,
    pub username: Option<String>,
    pub created_at: String,
}

impl TelegramAccount {
    pub async fn load(env: &Env, id: i64) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE id = ?1",
                DB_TABLE.telegram_account
            ))
            .bind(&[JsValue::from_f64(id as f64)])?
            .first::<TelegramAccount>(None)
            .await?
            .map(TelegramAccount::from)
            .ok_or(format!("Need to register (tg {id})").into())
    }

    pub async fn load_by_user_id(env: &Env, user_id: &UserId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE user_id = ?1",
                DB_TABLE.telegram_account
            ))
            .bind(&[user_id.into()])?
            .first::<TelegramAccount>(None)
            .await?
            .map(TelegramAccount::from)
            .ok_or(format!("Need to register (user_id {user_id})").into())
    }

    pub async fn update_name(
        env: &Env,
        id: i64,
        first_name: &str,
        username: Option<&str>,
    ) -> ApiResult<()> {
        match username {
            Some(username) => get_d1(env)?
                .prepare(format!(
                    "UPDATE {} SET first_name = ?1, username = ?2 WHERE id = ?3",
                    DB_TABLE.telegram_account
                ))
                .bind(&[
                    first_name.into(),
                    username.into(),
                    JsValue::from_f64(id as f64),
                ])?
                .run()
                .await?
                .into_result(),
            None => get_d1(env)?
                .prepare(format!(
                    "UPDATE {} SET first_name = ?1 WHERE id = ?2",
                    DB_TABLE.telegram_account
                ))
                .bind(&[first_name.into(), JsValue::from_f64(id as f64)])?
                .run()
                .await?
                .into_result(),
        }
    }

    pub async fn exists(env: &Env, id: i64) -> ApiResult<bool> {
        let res = get_d1(env)?
            .prepare(format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE id = ?1)",
                DB_TABLE.telegram_account
            ))
            .bind(&[JsValue::from_f64(id as f64)])?
            .raw::<u32>()
            .await?;

        let exists = res[0][0] == 1;

        Ok(exists)
    }

    pub async fn insert(env: &Env, id: i64, uid: &UserId) -> ApiResult<()> {
        // the real name and username will be updated later via messages
        get_d1(env)?
            .prepare(format!(
                "INSERT INTO {} (id, user_id, first_name) VALUES (?1, ?2, ?3)",
                DB_TABLE.telegram_account
            ))
            .bind(&[
                JsValue::from_f64(id as f64),
                uid.into(),
                format!("user {id}").into(),
            ])?
            .run()
            .await?
            .into_result()
    }
}
