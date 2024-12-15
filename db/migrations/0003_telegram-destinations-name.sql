-- Migration number: 0003 	 2024-11-30T22:02:03.304Z

ALTER TABLE telegram_destination
ADD COLUMN name TEXT NOT NULL DEFAULT 'default_name';
