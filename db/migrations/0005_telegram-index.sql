-- Migration number: 0005 	 2024-11-30T23:02:44.099Z
-- Index on telegram_destination.id (usually automatically created as it's a PRIMARY KEY)
CREATE INDEX IF NOT EXISTS idx_telegram_destination_id ON telegram_destination(id);

-- Index on telegram_destination.user_id
CREATE INDEX IF NOT EXISTS idx_telegram_destination_user_id ON telegram_destination(user_id);

-- Index on telegram_action.destination_id
CREATE INDEX IF NOT EXISTS idx_telegram_action_destination_id ON telegram_action(destination_id);