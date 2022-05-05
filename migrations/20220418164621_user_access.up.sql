CREATE TABLE user_access (
    client_id TEXT NOT NULL REFERENCES instances(client_id) ON DELETE CASCADE,
    email TEXT NOT NULL REFERENCES users(email) ON DELETE CASCADE,
    role TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    PRIMARY KEY (client_id, email)
);
CREATE INDEX user_access_client_id_idx ON user_access(client_id);
CREATE INDEX user_access_email_idx ON user_access(email);
