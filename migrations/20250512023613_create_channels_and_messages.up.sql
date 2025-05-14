CREATE TABLE hosts (
    domain TEXT PRIMARY KEY,
    user_count INT NOT NULL,
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

CREATE INDEX idx_messages_channel_id_created_at
    ON messages (channel_id, created_at);

CREATE FUNCTION set_updated_at()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Hosts and users are updated remotely and synced manually

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
