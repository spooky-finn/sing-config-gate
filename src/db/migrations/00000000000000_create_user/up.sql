CREATE TABLE user (
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    status INTEGER NOT NULL,
    auth_key TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE vpn_uuid (
    uuid TEXT PRIMARY KEY NOT NULL,
    user_id BIGINT NOT NULL REFERENCES user(id),
);

CREATE INDEX idx_vpn_uuid_uuid ON vpn_uuid(uuid);
CREATE INDEX idx_vpn_uuid_user_id ON vpn_uuid(user_id);
