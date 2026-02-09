-- Change reviewed_at from TIMESTAMPTZ to DATE.
-- Review tracking only needs day-level granularity. Using DATE prevents
-- tasks from progressively appearing during a review session (all tasks
-- reviewed on the same day share the same value).
ALTER TABLE tasks
    ALTER COLUMN reviewed_at TYPE DATE USING reviewed_at::date;

-- Default to current date so new tasks don't immediately appear in review.
ALTER TABLE tasks
    ALTER COLUMN reviewed_at SET DEFAULT CURRENT_DATE;

-- Backfill existing NULLs with created_at date.
UPDATE tasks SET reviewed_at = created_at::date WHERE reviewed_at IS NULL;
