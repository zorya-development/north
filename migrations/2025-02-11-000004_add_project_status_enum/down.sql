ALTER TABLE projects ADD COLUMN archived BOOLEAN NOT NULL DEFAULT false;
UPDATE projects SET archived = (status = 'archived');
ALTER TABLE projects DROP COLUMN status;
DROP TYPE project_status;
