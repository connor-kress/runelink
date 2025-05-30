CREATE TABLE user_associated_domains (
    user_id UUID NOT NULL
        REFERENCES users (id)
        ON DELETE CASCADE,
    domain TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, domain)
);

CREATE INDEX idx_user_associated_domains_user
    ON user_associated_domains (user_id);
