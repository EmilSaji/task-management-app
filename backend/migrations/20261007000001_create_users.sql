-- Users and roles
CREATE TYPE user_role AS ENUM ('admin', 'staff');

-- Keeps updated_at current on every UPDATE (shared by all tables).
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
    id              UUID PRIMARY KEY,
    full_name       TEXT        NOT NULL CHECK (length(trim(full_name)) > 0),
    email           TEXT        NOT NULL UNIQUE CHECK (email = lower(email)),
    hashed_password TEXT        NOT NULL,
    role            user_role   NOT NULL DEFAULT 'staff',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
