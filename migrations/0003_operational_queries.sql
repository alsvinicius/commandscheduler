CREATE INDEX occurrences_finished_status_idx ON occurrences(status, finished_at);
CREATE INDEX occurrences_created_page_idx ON occurrences(created_at DESC, id DESC);
