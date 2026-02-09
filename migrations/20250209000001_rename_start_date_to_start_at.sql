ALTER TABLE tasks RENAME COLUMN start_date TO start_at;
ALTER TABLE tasks ALTER COLUMN start_at TYPE TIMESTAMPTZ USING start_at::timestamptz;
