CREATE TYPE recurrence_type AS ENUM ('scheduled', 'after_completion');

ALTER TABLE tasks
  ADD COLUMN recurrence_type recurrence_type,
  ADD COLUMN recurrence_rule VARCHAR;
