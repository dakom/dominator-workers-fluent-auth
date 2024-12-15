-- Migration number: 0004 	 2024-11-30T22:27:06.366Z
CREATE TABLE telegram_action (
    id TEXT PRIMARY KEY,
    destination_id TEXT NOT NULL,
    prompt TEXT NOT NULL,
    msg TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;