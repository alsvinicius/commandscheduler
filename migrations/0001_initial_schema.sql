CREATE TABLE schedules (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT,
    program TEXT NOT NULL CHECK (length(trim(program)) > 0),
    args_json TEXT NOT NULL CHECK (json_valid(args_json) AND json_type(args_json) = 'array'),
    timing_json TEXT NOT NULL CHECK (json_valid(timing_json) AND json_type(timing_json) = 'object'),
    parameters_json TEXT NOT NULL DEFAULT 'null'
        CHECK (json_valid(parameters_json))
        CHECK (length(CAST(parameters_json AS BLOB)) <= 65536),
    missed_policy TEXT NOT NULL CHECK (missed_policy IN ('skip', 'consolidate', 'sequence')),
    parameter_update_trigger_json TEXT NOT NULL DEFAULT '{"enabled":false,"scheduleBehavior":"coexist"}'
        CHECK (json_valid(parameter_update_trigger_json)),
    retry_policy_json TEXT NOT NULL DEFAULT '{"maxRetries":3,"delaysSeconds":[60,300,900]}'
        CHECK (json_valid(retry_policy_json)),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'cancelled')),
    idempotency_key TEXT UNIQUE,
    idempotency_fingerprint TEXT CHECK (idempotency_fingerprint IS NULL OR length(idempotency_fingerprint) = 64),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK ((idempotency_key IS NULL) = (idempotency_fingerprint IS NULL))
);

CREATE INDEX schedules_status_idx ON schedules(status);

CREATE TABLE occurrences (
    id TEXT PRIMARY KEY NOT NULL,
    schedule_id TEXT NOT NULL REFERENCES schedules(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled', 'interrupted')),
    source TEXT NOT NULL CHECK (source IN ('schedule', 'parameter_update', 'manual', 'retry', 'recovery')),
    attempt INTEGER NOT NULL DEFAULT 1 CHECK (attempt >= 1),
    retry_of_id TEXT REFERENCES occurrences(id) ON DELETE SET NULL,
    scheduled_for INTEGER,
    created_at INTEGER NOT NULL,
    started_at INTEGER,
    finished_at INTEGER,
    exit_code INTEGER,
    stdout BLOB CHECK (stdout IS NULL OR length(stdout) <= 1048576),
    stderr BLOB CHECK (stderr IS NULL OR length(stderr) <= 1048576),
    stdout_truncated INTEGER NOT NULL DEFAULT 0 CHECK (stdout_truncated IN (0, 1)),
    stderr_truncated INTEGER NOT NULL DEFAULT 0 CHECK (stderr_truncated IN (0, 1)),
    missed_count INTEGER,
    missed_from INTEGER,
    missed_through INTEGER,
    CHECK (
        (missed_count IS NULL AND missed_from IS NULL AND missed_through IS NULL)
        OR (missed_count > 0 AND missed_from IS NOT NULL AND missed_through IS NOT NULL)
    )
);

CREATE INDEX occurrences_schedule_created_idx ON occurrences(schedule_id, created_at DESC);
CREATE INDEX occurrences_status_scheduled_idx ON occurrences(status, scheduled_for);
