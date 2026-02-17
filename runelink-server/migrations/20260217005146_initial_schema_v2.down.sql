-- DROP updated_at TRIGGERS
DROP TRIGGER IF EXISTS local_accounts_set_updated_at ON local_accounts;
DROP TRIGGER IF EXISTS server_users_set_updated_at ON server_users;
DROP TRIGGER IF EXISTS messages_set_updated_at ON messages;
DROP TRIGGER IF EXISTS channels_set_updated_at ON channels;
DROP TRIGGER IF EXISTS servers_set_updated_at ON servers;
DROP TRIGGER IF EXISTS users_set_updated_at ON users;

-- DROP TRIGGER FUNCTIONS
DROP FUNCTION IF EXISTS set_updated_at();

-- DROP INDEXES
DROP INDEX IF EXISTS idx_user_remote_memberships_remote_server_id;
DROP INDEX IF EXISTS idx_user_remote_memberships_user;
DROP INDEX IF EXISTS idx_messages_author;
DROP INDEX IF EXISTS idx_messages_channel_id_created_at;
DROP INDEX IF EXISTS idx_channels_server_id;

-- DROP TABLES
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS local_accounts;
DROP TABLE IF EXISTS user_remote_server_memberships;
DROP TABLE IF EXISTS cached_remote_servers;
DROP TABLE IF EXISTS server_users;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS channels;
DROP TABLE IF EXISTS servers;
DROP TABLE IF EXISTS users;

-- DROP TYPES
DROP TYPE IF EXISTS user_role;
DROP TYPE IF EXISTS server_role;

-- DROP EXTENSIONS
DROP EXTENSION IF EXISTS "pgcrypto";
