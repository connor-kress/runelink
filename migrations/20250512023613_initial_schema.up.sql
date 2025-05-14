-- TABLE DEFINITIONS --

CREATE TABLE users (
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,
    PRIMARY KEY (name, domain)
);

CREATE TABLE hosts (
    domain TEXT PRIMARY KEY,
    user_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_user_count_nonnegative
        CHECK (user_count >= 0)
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
    server_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_server
        FOREIGN KEY (server_id)
        REFERENCES servers (id)
        ON DELETE CASCADE
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL,
    author_name TEXT,
    author_domain TEXT,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_channel
        FOREIGN KEY (channel_id)
        REFERENCES channels (id)
        ON DELETE CASCADE,

    CONSTRAINT fk_author
        FOREIGN KEY (author_name, author_domain)
        REFERENCES users (name, domain)
        ON DELETE SET NULL
        ON UPDATE CASCADE,

    -- All or nothing for the author
    CONSTRAINT chk_author_fields_complete_or_null CHECK (
        (author_name IS NULL AND author_domain IS NULL) OR
        (author_name IS NOT NULL AND author_domain IS NOT NULL)
    )
);


-- INDEXES FOR COMMON ACCESSES --

CREATE INDEX idx_channels_server_id
    ON channels (server_id);

CREATE INDEX idx_messages_channel_id_created_at
    ON messages (channel_id, created_at);

CREATE INDEX idx_messages_author
    ON messages (author_name, author_domain);


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
