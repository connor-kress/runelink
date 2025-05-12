CREATE TABLE messages (
    id UUID PRIMARY KEY,
    sender_name TEXT,
    sender_domain TEXT,
    recipient_name TEXT,
    recipient_domain TEXT,
    body TEXT NOT NULL,

    CONSTRAINT fk_sender
        FOREIGN KEY (sender_name, sender_domain)
        REFERENCES users (name, domain)
        ON DELETE SET NULL
        ON UPDATE CASCADE,

    CONSTRAINT fk_recipient
        FOREIGN KEY (recipient_name, recipient_domain)
        REFERENCES users (name, domain)
        ON DELETE SET NULL
        ON UPDATE CASCADE,

    -- Sender all or nothing
    CONSTRAINT chk_sender_fields_complete_or_null CHECK (
        (sender_name IS NULL AND sender_domain IS NULL) OR
        (sender_name IS NOT NULL AND sender_domain IS NOT NULL)
    ),

    -- Recipient all or nothing
    CONSTRAINT chk_recipient_fields_complete_or_null CHECK (
        (recipient_name IS NULL AND recipient_domain IS NULL) OR
        (recipient_name IS NOT NULL AND recipient_domain IS NOT NULL)
    )
);
