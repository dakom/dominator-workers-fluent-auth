-- Migration number: 0001 	 2024-11-26T12:24:01.929Z
CREATE TABLE user_account (
    id TEXT PRIMARY KEY,
	user_token TEXT NOT NULL
) WITHOUT ROWID;


CREATE TABLE user_account_email (
    email TEXT PRIMARY KEY,
    password TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES user_account(id)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_email_user_id ON user_account_email(user_id);

CREATE TABLE user_role_info (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

INSERT INTO user_role_info (name, description) VALUES ('zero_index', 'Zero-index placeholder');
INSERT INTO user_role_info (name, description) VALUES ('basic', 'Basic user');
INSERT INTO user_role_info (name, description) VALUES ('email_verified', 'Email verified');
INSERT INTO user_role_info (name, description) VALUES ('admin', 'Administrator');

CREATE TABLE user_roles (
    user_id TEXT NOT NULL,
    role_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES user_account(id),
    FOREIGN KEY (role_id) REFERENCES user_role_info(id)
);