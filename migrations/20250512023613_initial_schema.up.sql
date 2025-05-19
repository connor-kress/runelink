-- TABLE DEFINITIONS --

CREATE TABLE users (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,
    UNIQUE (name, domain)
);

CREATE TABLE hosts (
    domain TEXT PRIMARY KEY,
    user_count INT NOT NULL DEFAULT 0
        CHECK (user_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE servers (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE channels (
    id UUID PRIMARY KEY,
    server_id UUID NOT NULL
        REFERENCES servers (id)
        ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL
        REFERENCES channels (id)
        ON DELETE CASCADE,
    author_id UUID
        REFERENCES users (id)
        ON DELETE SET NULL
        ON UPDATE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- INDEXES FOR COMMON ACCESSES --

CREATE INDEX idx_channels_server_id
    ON channels (server_id);

CREATE INDEX idx_messages_channel_id_created_at
    ON messages (channel_id, created_at);

CREATE INDEX idx_messages_author
    ON messages (author_id);


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

CREATE TRIGGER hosts_set_updated_at
    BEFORE UPDATE ON hosts
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


-- USER_COUNT MANAGEMENT --

-- Increment user_count on user insertions
CREATE OR REPLACE FUNCTION increment_host_user_count()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO hosts(domain, user_count)
        VALUES (NEW.domain, 1)
    ON CONFLICT (domain)
        DO UPDATE SET user_count = hosts.user_count + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_after_insert
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION increment_host_user_count();

-- Decrement user_count on users deletions
CREATE OR REPLACE FUNCTION decrement_host_user_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE hosts
    SET user_count = user_count - 1
    WHERE domain = OLD.domain;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_after_delete
    AFTER DELETE ON users
    FOR EACH ROW
    EXECUTE FUNCTION decrement_host_user_count();

-- Adjust user_counts on user updates
CREATE OR REPLACE FUNCTION adjust_host_user_count_on_update()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.domain <> OLD.domain THEN
        -- decrement old domain
        UPDATE hosts
        SET user_count = user_count - 1
        WHERE domain = OLD.domain;

        -- increment new domain
        UPDATE hosts
        SET user_count = user_count + 1
        WHERE domain = NEW.domain;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_after_update
    AFTER UPDATE OF domain ON users
    FOR EACH ROW
    EXECUTE FUNCTION adjust_host_user_count_on_update();
