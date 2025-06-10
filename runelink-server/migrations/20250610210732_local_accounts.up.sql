CREATE TABLE local_accounts (
    user_id UUID PRIMARY KEY
        REFERENCES users(id),
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER local_accounts_set_updated_at
    BEFORE UPDATE ON local_accounts
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
