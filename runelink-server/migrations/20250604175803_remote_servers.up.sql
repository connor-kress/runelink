-- Remote servers that local users are members of
CREATE TABLE cached_remote_servers (
    id UUID PRIMARY KEY,
    domain TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    remote_created_at TIMESTAMPTZ NOT NULL,
    remote_updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Local user memberships in remote servers
CREATE TABLE user_remote_server_memberships (
    user_id UUID NOT NULL
        REFERENCES users (id)
        ON DELETE CASCADE,
    remote_server_id UUID NOT NULL
        REFERENCES cached_remote_servers (id)
        ON DELETE CASCADE,
    role server_role NOT NULL,
    remote_created_at TIMESTAMPTZ NOT NULL,
    remote_updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, remote_server_id)
);

CREATE INDEX IF NOT EXISTS idx_user_remote_memberships_user_id
    ON user_remote_server_memberships (user_id);

CREATE INDEX IF NOT EXISTS idx_user_remote_memberships_remote_server_id
    ON user_remote_server_memberships (remote_server_id);
