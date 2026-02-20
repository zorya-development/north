ALTER TABLE tasks
  DROP COLUMN recurrence_rule,
  DROP COLUMN recurrence_type;

DROP TYPE recurrence_type;
