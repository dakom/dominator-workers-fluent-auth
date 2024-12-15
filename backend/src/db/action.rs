use crate::{config::DB_TABLE, prelude::*};
use serde::{Deserialize, Serialize};
use shared::{
    api::action::{
        Action, ActionDestination, ActionDestinationId, ActionDestinationKind, ActionId,
    },
    user::UserId,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramActionDb {
    pub id: ActionId,
    pub destination_id: ActionDestinationId,
    pub prompt: String,
    pub msg: String,
    pub created_at: String,
}

impl TelegramActionDb {
    pub async fn _load(env: &Env, id: &ActionId) -> ApiResult<Self> {
        get_d1(env)?
            .prepare(format!(
                "SELECT * FROM {} WHERE id = ?1",
                DB_TABLE.telegram_action
            ))
            .bind(&[id.into()])?
            .first::<Self>(None)
            .await?
            .map(Self::from)
            .ok_or(format!("no such action with id {id}").into())
    }

    pub async fn insert(
        env: &Env,
        id: &ActionId,
        destination_id: &ActionDestinationId,
        prompt: &str,
        msg: &str,
    ) -> ApiResult<()> {
        get_d1(env)?
            .prepare(format!(
                "INSERT INTO {} (id, destination_id, prompt, msg) VALUES (?1, ?2, ?3, ?4)",
                DB_TABLE.telegram_action
            ))
            .bind(&[id.into(), destination_id.into(), prompt.into(), msg.into()])?
            .run()
            .await?
            .into_result()
    }

    pub async fn delete(env: &Env, user_id: &UserId, id: &ActionId) -> ApiResult<()> {
        let stmt = format!(
            r#"
            DELETE FROM {} 
            WHERE id = ?1 
            AND destination_id IN (
                SELECT id FROM {} WHERE user_id = ?2 
            )
        "#,
            DB_TABLE.telegram_action, DB_TABLE.telegram_destination
        );

        get_d1(env)?
            .prepare(stmt)
            .bind(&[id.into(), user_id.into()])?
            .run()
            .await?
            .into_result()
    }

    pub async fn list(env: &Env, user_id: &UserId) -> ApiResult<Vec<Action>> {
        #[derive(Deserialize, Serialize, Debug)]
        pub struct JoinedRecord {
            pub id: ActionId,
            pub destination_id: ActionDestinationId,
            pub prompt: String,
            pub msg: String,
            pub name: String,
            pub created_at: String,
            pub chat_id: i64,
            pub kind: u8,
        }

        let stmt = format!(
            r#"
            SELECT ta.*, td.chat_id, td.kind, td.name
            FROM {} AS ta
            JOIN {} AS td ON ta.destination_id = td.id
            WHERE td.user_id = ?1
        "#,
            DB_TABLE.telegram_action, DB_TABLE.telegram_destination
        );

        Ok(get_d1(env)?
            .prepare(stmt)
            .bind(&[user_id.into()])?
            .all()
            .await?
            .results::<JoinedRecord>()?
            .into_iter()
            .map(|r| Action {
                id: r.id,
                destination: ActionDestination {
                    id: r.destination_id,
                    name: r.name,
                    kind: match r.kind {
                        1 => ActionDestinationKind::TelegramDm { chat_id: r.chat_id },
                        2 => ActionDestinationKind::TelegramGroup { chat_id: r.chat_id },
                        _ => unreachable!(),
                    },
                },
                prompt: r.prompt,
                message: r.msg,
            })
            .collect())
    }
}

// CREATE TABLE telegram_action (
//     id TEXT PRIMARY KEY,
//     destination_id TEXT NOT NULL,
//     prompt TEXT NOT NULL,
//     msg TEXT NOT NULL,
//     created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
// ) WITHOUT ROWID;
