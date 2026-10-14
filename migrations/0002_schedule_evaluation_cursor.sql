ALTER TABLE schedules ADD COLUMN last_evaluated_at INTEGER;

UPDATE schedules SET last_evaluated_at = created_at WHERE last_evaluated_at IS NULL;

CREATE INDEX schedules_evaluation_idx ON schedules(status, last_evaluated_at);
