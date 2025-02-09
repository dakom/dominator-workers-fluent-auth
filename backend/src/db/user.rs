use crate::{config::DB_TABLE, prelude::*};
use serde::{Deserialize, Serialize};
use shared::{api::auth::UserRole, user::UserId};

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccountDb {
    pub id: UserId,
    pub user_token: String,
}

pub enum UserInsertKind<'a> {
    EmailPw { email: &'a str, password: &'a str },
}

impl UserAccountDb {
    pub async fn load(env: &Env, id: &UserId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE id = ?1",
                DB_TABLE.user_account
            ))
            .bind(&[id.into()])?
            .first::<UserAccountDb>(None)
            .await?
            .map(UserAccountDb::from)
            .ok_or(format!("Need to register (id {id})").into())
    }

    pub async fn _exists(env: &Env, id: &UserId) -> ApiResult<bool> {
        let res = get_d1(env)?
            .prepare(format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE id = ?1)",
                DB_TABLE.user_account
            ))
            .bind(&[id.into()])?
            .raw::<u32>()
            .await?;

        let exists = res[0][0] == 1;

        Ok(exists)
    }

    pub async fn insert<'a>(
        env: &Env,
        id: &UserId,
        user_token: &str,
        kind: UserInsertKind<'a>,
        roles: Vec<UserRole>,
    ) -> ApiResult<()> {
        let d1 = get_d1(env)?;

        let mut statements = vec![
            d1.prepare(format!(
                "INSERT INTO {} (id, user_token) VALUES (?1, ?2)",
                DB_TABLE.user_account
            ))
            .bind(&[id.into(), user_token.into()])?,
            match kind {
                UserInsertKind::EmailPw { email, password } => d1
                    .prepare(format!(
                        "INSERT INTO {} (email, password, user_id) VALUES (?1, ?2, ?3)",
                        DB_TABLE.user_account_email
                    ))
                    .bind(&[email.into(), password.into(), id.into()])?,
            },
        ];

        if !roles.is_empty() {
            for role in roles {
                statements.push(
                    d1.prepare(format!(
                        "INSERT INTO {} (user_id, role_id) VALUES (?1, ?2)",
                        DB_TABLE.user_roles
                    ))
                    .bind(&[id.into(), u8::from(role).into()])?,
                );
            }
        }

        let res = d1.batch(statements).await?;

        for r in res {
            if let Some(err) = r.error() {
                return Err(err.into());
            }
        }

        Ok(())
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

    pub async fn load_roles(env: &Env, id: &UserId) -> ApiResult<Vec<UserRole>> {
        #[derive(Deserialize, Serialize, Debug)]
        struct RoleDb {
            role_id: u8,
        }

        let res = get_d1(env)?
            .prepare(format!(
                "SELECT role_id FROM {} WHERE user_id = ?1",
                DB_TABLE.user_roles
            ))
            .bind(&[id.into()])?
            .all()
            .await?
            .results::<RoleDb>()?;

        let res = res.into_iter().map(|r| UserRole::from(r.role_id)).collect();

        Ok(res)
    }

    pub async fn add_role(env: &Env, id: &UserId, role: UserRole) -> ApiResult<()> {
        get_d1(env)?
            .prepare(format!(
                "INSERT INTO {} (user_id, role_id) VALUES (?1, ?2)",
                DB_TABLE.user_roles
            ))
            .bind(&[id.into(), u8::from(role).into()])?
            .run()
            .await?
            .into_result()
    }
}

// no insert for user_email, since it's created at the same time as user_account
#[derive(Deserialize, Serialize, Debug)]
pub struct UserEmailDb {
    pub email: String,
    pub password: String,
    pub user_id: UserId,
    pub created_at: String,
}

impl UserEmailDb {
    pub async fn load(env: &Env, email: &str) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE email = ?1",
                DB_TABLE.user_account_email
            ))
            .bind(&[email.into()])?
            .first::<UserEmailDb>(None)
            .await?
            .map(UserEmailDb::from)
            .ok_or(format!("Need to register (email {email})").into())
    }

    pub async fn exists(env: &Env, email: &str) -> ApiResult<bool> {
        let res = get_d1(env)?
            .prepare(format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE email = ?1)",
                DB_TABLE.user_account_email
            ))
            .bind(&[email.into()])?
            .raw::<u32>()
            .await?;

        let exists = res[0][0] == 1;

        Ok(exists)
    }

    pub async fn load_by_user_id(env: &Env, user_id: &UserId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE user_id = ?1",
                DB_TABLE.user_account_email
            ))
            .bind(&[user_id.into()])?
            .first::<UserEmailDb>(None)
            .await?
            .map(UserEmailDb::from)
            .ok_or(format!("Need to register (user_id {user_id})").into())
    }
}
