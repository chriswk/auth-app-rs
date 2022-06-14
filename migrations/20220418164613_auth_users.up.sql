CREATE TABLE auth_users (
    email TEXT PRIMARY KEY NOT NULL,
    name TEXT,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    password_reset_token TEXT,
    password_reset_expires TIMESTAMP WITH TIME ZONE,
    login_attempts INTEGER
);
