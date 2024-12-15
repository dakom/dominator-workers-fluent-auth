-- Migration number: 0006 	 2024-12-01T00:36:50.010Z

ALTER TABLE telegram_account
ADD COLUMN first_name TEXT NOT NULL DEFAULT 'default_name';

ALTER TABLE telegram_account
ADD COLUMN username TEXT;