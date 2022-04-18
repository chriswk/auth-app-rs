CREATE TABLE history (
    id BIGSERIAL,
    email TEXT NOT NULL,
    action TEXT NOT NULL,
    ip TEXT,
    message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);
