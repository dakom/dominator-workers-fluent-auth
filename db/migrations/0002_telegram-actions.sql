-- Migration number: 0002 	 2024-11-30T21:06:07.420Z
CREATE TABLE telegram_destination (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    chat_id BIGINT NOT NULL,
    kind INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;

-- convert telegram account type

CREATE TABLE telegram_account_new (
    id BIGINT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;

INSERT INTO telegram_account_new (id, user_id, created_at)
SELECT CAST(id AS BIGINT), user_id, created_at
FROM telegram_account;

DROP TABLE telegram_account;

ALTER TABLE telegram_account_new RENAME TO telegram_account;