-- Add migration script here
CREATE TABLE files (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bucket_id   UUID NOT NULL REFERENCES buckets(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    mime_type   TEXT NOT NULL,
    size        BIGINT NOT NULL,        -- bytes
    path        TEXT NOT NULL UNIQUE,   -- where it lives on disk e.g. "ab/cd/abcd1234"
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);