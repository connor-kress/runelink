CREATE TYPE server_role AS ENUM ('member', 'admin');

CREATE TABLE server_users (
    user_id UUID NOT NULL
        REFERENCES users (id)
        ON DELETE CASCADE,
    server_id UUID NOT NULL
        REFERENCES servers (id)
        ON DELETE CASCADE,
    role server_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, server_id)
);

CREATE TRIGGER server_users_set_updated_at
    BEFORE UPDATE ON server_users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
