DROP INDEX "requests_path_idx";--> statement-breakpoint
CREATE INDEX "requests_status_idx" ON "requests" USING btree ("status") WHERE status = 200;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS idx_requests_builds_type
ON requests ((data->>'type'))
WHERE data->>'type' = 'builds';--> statement-breakpoint
CREATE INDEX IF NOT EXISTS idx_requests_search_type
ON requests ((data->'search'->>'type'))
WHERE data->>'type' = 'builds';--> statement-breakpoint
CREATE INDEX IF NOT EXISTS idx_requests_search_type_version
ON requests ((data->'search'->>'type'), (data->'search'->>'version'))
WHERE data->>'type' = 'builds';--> statement-breakpoint
CREATE INDEX idx_requests_search_version
ON requests ((data->'search'->>'version'))
WHERE data->>'type' = 'builds' AND data->'search'->>'version' IS NOT NULL;