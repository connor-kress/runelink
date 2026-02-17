-- EXTENSIONS --

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- TYPES --

CREATE TYPE server_role AS ENUM ('member', 'admin');
CREATE TYPE user_role AS ENUM ('user', 'admin');

-- TABLE DEFINITIONS --

CREATE TABLE users (
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,
    PRIMARY KEY (name, host)
);

CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL
        REFERENCES servers (id)
        ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL
        REFERENCES channels (id)
        ON DELETE CASCADE,
    author_name TEXT,
    author_host TEXT,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT messages_author_fkey
        FOREIGN KEY (author_name, author_host)
        REFERENCES users(name, host)
        ON DELETE SET NULL
);

CREATE TABLE server_users (
    user_name TEXT NOT NULL,
    user_host TEXT NOT NULL,
    server_id UUID NOT NULL
        REFERENCES servers (id)
        ON DELETE CASCADE,
    role server_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_name, user_host, server_id),
    CONSTRAINT server_users_user_fkey
        FOREIGN KEY (user_name, user_host)
        REFERENCES users(name, host)
        ON DELETE CASCADE
);

-- Remote servers that local users are members of
CREATE TABLE cached_remote_servers (
    id UUID PRIMARY KEY,
    host TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    remote_created_at TIMESTAMPTZ NOT NULL,
    remote_updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Local user memberships in remote servers
CREATE TABLE user_remote_server_memberships (
    user_name TEXT NOT NULL,
    user_host TEXT NOT NULL,
    remote_server_id UUID NOT NULL
        REFERENCES cached_remote_servers (id)
        ON DELETE CASCADE,
    role server_role NOT NULL,
    remote_created_at TIMESTAMPTZ NOT NULL,
    remote_updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_name, user_host, remote_server_id),
    CONSTRAINT user_remote_server_memberships_user_fkey
        FOREIGN KEY (user_name, user_host)
        REFERENCES users(name, host)
        ON DELETE CASCADE
);

CREATE TABLE local_accounts (
    user_name TEXT NOT NULL,
    user_host TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_name, user_host),
    CONSTRAINT local_accounts_user_fkey
        FOREIGN KEY (user_name, user_host)
        REFERENCES users(name, host)
        ON DELETE CASCADE
);

-- For local accounts
CREATE TABLE refresh_tokens (
    token TEXT PRIMARY KEY,
    user_name TEXT NOT NULL,
    user_host TEXT NOT NULL,
    client_id TEXT NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT refresh_tokens_user_fkey
        FOREIGN KEY (user_name, user_host)
        REFERENCES users(name, host)
        ON DELETE CASCADE
);

-- INDEXES FOR COMMON ACCESSES --

CREATE INDEX idx_channels_server_id
    ON channels (server_id);

CREATE INDEX idx_messages_channel_id_created_at
    ON messages (channel_id, created_at);

CREATE INDEX idx_messages_author
    ON messages (author_name, author_host)
    WHERE author_name IS NOT NULL;

CREATE INDEX idx_user_remote_memberships_user
    ON user_remote_server_memberships (user_name, user_host);

CREATE INDEX idx_user_remote_memberships_remote_server_id
    ON user_remote_server_memberships (remote_server_id);

-- UPDATED_AT MANAGEMENT --

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER servers_set_updated_at
    BEFORE UPDATE ON servers
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER channels_set_updated_at
    BEFORE UPDATE ON channels
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER messages_set_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER server_users_set_updated_at
    BEFORE UPDATE ON server_users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER local_accounts_set_updated_at
    BEFORE UPDATE ON local_accounts
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
