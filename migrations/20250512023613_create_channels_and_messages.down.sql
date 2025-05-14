DROP TRIGGER IF EXISTS messages_set_updated_at ON messages;
DROP TRIGGER IF EXISTS channels_set_updated_at ON channels;
DROP TRIGGER IF EXISTS servers_set_updated_at ON servers;
DROP FUNCTION IF EXISTS set_updated_at();
DROP INDEX IF EXISTS idx_messages_channel_id_created_at;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS channels;
DROP TABLE IF EXISTS servers;
DROP TABLE IF EXISTS hosts;
