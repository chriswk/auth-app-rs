CREATE TABLE user_access (
    client_id TEXT REFERENCES instances(client_id) ON DELETE CASCADE,
    email TEXT REFERENCES users(email) ON DELETE CASCADE,
    role TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);
