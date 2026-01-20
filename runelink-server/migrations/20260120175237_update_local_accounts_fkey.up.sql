-- Add `ON DELETE CASCADE` to the foreign key constraint
ALTER TABLE local_accounts
DROP CONSTRAINT local_accounts_user_id_fkey,
ADD CONSTRAINT local_accounts_user_id_fkey
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE;
