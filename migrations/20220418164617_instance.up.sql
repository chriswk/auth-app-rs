CREATE TABLE instances (
    client_id TEXT PRIMARY KEY NOT NULL,
    plan TEXT NOT NULL,
    display_name TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);
