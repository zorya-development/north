ALTER TABLE tasks ADD COLUMN sort_key VARCHAR NOT NULL DEFAULT '';
UPDATE tasks SET sort_key = lpad(position::text, 10, '0');
ALTER TABLE tasks DROP COLUMN position;
