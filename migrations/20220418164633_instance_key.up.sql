CREATE TABLE instance_keys (
    key TEXT PRIMARY KEY NOT NULL,
    client_id TEXT REFERENCES instances(client_id) ON DELETE CASCADE,
    active BOOLEAN,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);
