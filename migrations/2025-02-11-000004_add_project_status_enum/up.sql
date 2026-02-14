CREATE TYPE project_status AS ENUM ('active', 'archived');
ALTER TABLE projects ADD COLUMN status project_status NOT NULL DEFAULT 'active';
UPDATE projects SET status = CASE WHEN archived THEN 'archived'::project_status ELSE 'active'::project_status END;
ALTER TABLE projects DROP COLUMN archived;
