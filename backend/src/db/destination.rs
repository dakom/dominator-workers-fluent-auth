use crate::{config::DB_TABLE, prelude::*};
use serde::{Deserialize, Serialize};
use shared::{
    api::action::{ActionDestination, ActionDestinationId, ActionDestinationKind},
    user::UserId,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramDestinationDb {
    pub id: ActionDestinationId,
    pub name: String,
    pub user_id: UserId,
    pub chat_id: i64,
    pub kind: u8,
    pub created_at: String,
}

impl From<TelegramDestinationDb> for ActionDestination {
    fn from(u: TelegramDestinationDb) -> Self {
        let kind = match u.kind {
            1 => ActionDestinationKind::TelegramDm { chat_id: u.chat_id },
            2 => ActionDestinationKind::TelegramGroup { chat_id: u.chat_id },
            _ => unreachable!(),
        };

        ActionDestination {
            id: u.id,
            kind,
            name: u.name,
        }
    }
}

impl TelegramDestinationDb {
    pub async fn load(env: &Env, id: &ActionDestinationId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE id = ?1",
                DB_TABLE.telegram_destination
            ))
            .bind(&[id.into()])?
            .first::<Self>(None)
            .await?
            .map(Self::from)
            .ok_or(format!("no such destination with id {id}").into())
    }

    pub async fn load_with_user_id(
        env: &Env,
        id: &ActionDestinationId,
        user_id: &UserId,
    ) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE id = ?1 AND user_id = ?2",
                DB_TABLE.telegram_destination
            ))
            .bind(&[id.into(), user_id.into()])?
            .first::<Self>(None)
            .await?
            .map(Self::from)
            .ok_or(format!("no such destination with id {id} and user_id {user_id}").into())
    }

    pub async fn exists_by_user_chat_id(
        env: &Env,
        user_id: &UserId,
        chat_id: i64,
    ) -> ApiResult<bool> {
        let res = get_d1(env)?
            .prepare(format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE user_id = ?1 AND chat_id = ?2)",
                DB_TABLE.telegram_destination
            ))
            .bind(&[user_id.into(), JsValue::from_f64(chat_id as f64)])?
            .raw::<u32>()
            .await?;

        let exists = res[0][0] == 1;

        Ok(exists)
    }

    pub async fn insert(
        env: &Env,
        id: &ActionDestinationId,
        user_id: &UserId,
        name: &str,
        destination: ActionDestinationKind,
    ) -> ApiResult<()> {
        let (kind, chat_id) = match destination {
            ActionDestinationKind::TelegramDm { chat_id } => (1, chat_id),
            ActionDestinationKind::TelegramGroup { chat_id } => (2, chat_id),
        };

        get_d1(env)?
            .prepare(format!(
                "INSERT INTO {} (id, user_id, chat_id, name, kind) VALUES (?1, ?2, ?3, ?4, ?5)",
                DB_TABLE.telegram_destination
            ))
            .bind(&[
                id.into(),
                user_id.into(),
                JsValue::from_f64(chat_id as f64),
                name.into(),
                kind.into(),
            ])?
            .run()
            .await?
            .into_result()
    }

    pub async fn list(env: &Env, user_id: &UserId) -> ApiResult<Vec<ActionDestination>> {
        Ok(get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE user_id = ?1",
                DB_TABLE.telegram_destination
            ))
            .bind(&[user_id.into()])?
            .all()
            .await?
            .results::<Self>()?
            .into_iter()
            .map(ActionDestination::from)
            .collect())
    }
}
