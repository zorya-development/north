CREATE TABLE saved_filters (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    query TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_saved_filters_user_id ON saved_filters(user_id);

CREATE TRIGGER saved_filters_updated_at
    BEFORE UPDATE ON saved_filters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
