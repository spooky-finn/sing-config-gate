-- Your SQL goes here
CREATE TABLE user (
    id BIGINT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE vless_identity (
    uuid TEXT PRIMARY KEY NOT NULL,
    user_id BIGINT REFERENCES user(id)
);

CREATE INDEX idx_vless_identity_uuid ON vless_identity(uuid);
CREATE INDEX idx_vless_identity_user_id ON vless_identity(user_id);
