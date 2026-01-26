-- DROP USER_COUNT TRIGGERS
DROP TRIGGER IF EXISTS users_after_insert ON users;
DROP TRIGGER IF EXISTS users_after_delete ON users;
DROP TRIGGER IF EXISTS users_after_update ON users;

-- DROP updated_at TRIGGERS
DROP TRIGGER IF EXISTS messages_set_updated_at ON messages;
DROP TRIGGER IF EXISTS channels_set_updated_at ON channels;
DROP TRIGGER IF EXISTS servers_set_updated_at ON servers;
DROP TRIGGER IF EXISTS hosts_set_updated_at ON hosts;
DROP TRIGGER IF EXISTS users_set_updated_at ON users;

-- DROP TRIGGER FUNCTIONS
DROP FUNCTION IF EXISTS increment_host_user_count();
DROP FUNCTION IF EXISTS decrement_host_user_count();
DROP FUNCTION IF EXISTS adjust_host_user_count_on_update();
DROP FUNCTION IF EXISTS set_updated_at();

-- DROP INDEXES
DROP INDEX IF EXISTS idx_messages_author;
DROP INDEX IF EXISTS idx_messages_channel_id_created_at;
DROP INDEX IF EXISTS idx_channels_server_id;

-- DROP TABLES
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS channels;
DROP TABLE IF EXISTS servers;
DROP TABLE IF EXISTS hosts;
DROP TABLE IF EXISTS users;

-- DROP EXTENSIONS
DROP EXTENSION IF EXISTS "pgcrypto";
