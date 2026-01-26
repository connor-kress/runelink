-- Host-wide user roles (separate from server member roles)
CREATE TYPE user_role AS ENUM ('user', 'admin');

ALTER TABLE users
ADD COLUMN role user_role NOT NULL DEFAULT 'user';
