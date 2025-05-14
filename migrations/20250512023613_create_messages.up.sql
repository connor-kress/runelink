CREATE TABLE messages (
    id UUID PRIMARY KEY,
    sender_name TEXT,
    sender_domain TEXT,
    recipient_name TEXT,
    recipient_domain TEXT,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

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

CREATE FUNCTION set_updated_at()
  RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER messages_set_updated_at
  BEFORE UPDATE ON messages
  FOR EACH ROW
  EXECUTE FUNCTION set_updated_at();
