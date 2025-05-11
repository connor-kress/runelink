CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    UNIQUE (name, domain)
);
