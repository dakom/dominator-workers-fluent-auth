use serde::{Deserialize, Serialize};
use shared::user::UserId;
use crate::{
    config::DB_TABLE,
    prelude::*
};

#[derive(Deserialize, Serialize, Debug)]
struct UserAccountDb{
    pub id: UserId,
    pub password: String,
    pub email: String,
    pub email_verified: DbBool,
    pub user_token: String,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccount{
    pub id: UserId,
    pub password: String,
    pub email: String,
    pub email_verified: bool,
    pub user_token: String,
    pub created_at: String,
}

impl From<UserAccountDb> for UserAccount {
    fn from(db: UserAccountDb) -> Self {
        Self {
            id: db.id,
            password: db.password,
            email: db.email,
            email_verified: db.email_verified.into(),
            user_token: db.user_token,
            created_at: db.created_at,
        }
    }
}

impl UserAccount {
    pub async fn load_by_id(env: &Env, uid: &UserId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!("SELECT * FROM {} WHERE id = ?1", DB_TABLE.user_account))
            .bind(&[uid.into()])?
            .first::<UserAccountDb>(None).await?
            .map(UserAccount::from)
            .ok_or(format!("no such user with uid {uid}").into())
    }
    pub async fn load_by_email(env: &Env, email: &str) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!("SELECT * FROM {} WHERE email = ?1", DB_TABLE.user_account))
            .bind(&[email.into()])?
            .first::<UserAccountDb>(None).await?
            .map(UserAccount::from)
            .ok_or(format!("no such user with email {email}").into())
    }

    pub async fn exists_by_email(env: &Env, email: &str) -> ApiResult<bool> {
        let res = get_d1(env)?
            .prepare(format!("SELECT EXISTS(SELECT 1 FROM {} WHERE email = ?1)", DB_TABLE.user_account))
            .bind(&[email.into()])?
            .raw::<u32>()
            .await?;

        let exists = res[0][0] == 1;

        Ok(exists)
    }

    pub async fn insert(env: &Env, uid: &UserId, password: &str, email: &str, user_token: &str) -> ApiResult<()> {
        get_d1(env)?
            .prepare(format!("INSERT INTO {} (id, password, email, user_token) VALUES (?1, ?2, ?3, ?4)", DB_TABLE.user_account))
            .bind(&[uid.into(), password.into(), email.into(), user_token.into()])?
            .run()
            .await?
            .into_result()
    }

    pub async fn update_email_verified(env: &Env, uid: &UserId, verified: bool) -> ApiResult<()> {
        let verified = DbBool::from(verified);

        get_d1(env)?
            .prepare(format!("UPDATE {} SET email_verified = ?1 WHERE id = ?2", DB_TABLE.user_account))
            .bind(&[verified.into(), uid.into()])?
            .run()
            .await?
            .into_result()
    }

    pub async fn reset_password(env: &Env, uid: &UserId, password: &str, user_token: &str) -> ApiResult<()> {
        get_d1(env)?
            .prepare(format!("UPDATE {} SET password = ?1, user_token = ?2 WHERE id = ?3", DB_TABLE.user_account))
            .bind(&[password.into(), user_token.into(), uid.into()])?
            .run()
            .await?
            .into_result()
    }

}