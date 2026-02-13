ALTER TABLE tasks ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
UPDATE tasks SET position = CAST(ltrim(sort_key, '0') AS INTEGER);
ALTER TABLE tasks DROP COLUMN sort_key;
