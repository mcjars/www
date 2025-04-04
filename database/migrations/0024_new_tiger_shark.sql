DROP INDEX "requests_data_idx";--> statement-breakpoint

CREATE MATERIALIZED VIEW mv_requests_stats AS
SELECT
	data->>'type' AS request_type,
	data->'search'->>'type' AS search_type,
	data->'search'->>'version' AS search_version,
	data->'build'->>'type' AS build_type,
	data->'build'->>'versionId' AS build_version_id,
	data->'build'->>'projectVersionId' AS build_project_version_id,
	COUNT(*)::BIGINT AS total_requests,
	COUNT(DISTINCT ip)::BIGINT AS unique_ips,
	MIN(created) AS first_seen,
	MAX(created) AS last_seen
FROM requests
WHERE 
	data IS NOT NULL 
	AND status = 200
	AND path NOT LIKE '%tracking=nostats%'
GROUP BY 
	data->>'type',
	data->'search'->>'type',
	data->'search'->>'version',
	data->'build'->>'type',
	data->'build'->>'versionId',
	data->'build'->>'projectVersionId';--> statement-breakpoint

CREATE INDEX idx_mv_stats_request_type ON mv_requests_stats(request_type);--> statement-breakpoint
CREATE INDEX idx_mv_stats_search_type ON mv_requests_stats(search_type);--> statement-breakpoint
CREATE INDEX idx_mv_stats_search_version ON mv_requests_stats(search_version);--> statement-breakpoint
CREATE INDEX idx_mv_stats_build_type ON mv_requests_stats(build_type);--> statement-breakpoint
CREATE INDEX idx_mv_stats_build_version_id ON mv_requests_stats(build_version_id);--> statement-breakpoint
CREATE INDEX idx_mv_stats_build_project_version_id ON mv_requests_stats(build_project_version_id);--> statement-breakpoint

CREATE INDEX idx_mv_stats_request_search_type ON mv_requests_stats(request_type, search_type);--> statement-breakpoint
CREATE INDEX idx_mv_stats_request_search_version ON mv_requests_stats(request_type, search_version);--> statement-breakpoint
CREATE INDEX idx_mv_stats_request_build_version ON mv_requests_stats(request_type, build_version_id);--> statement-breakpoint

CREATE MATERIALIZED VIEW mv_requests_stats_daily AS
WITH request_data AS (
	SELECT
		data->>'type' AS request_type,
		data->'search'->>'type' AS search_type,
		data->'search'->>'version' AS search_version,
		data->'build'->>'type' AS build_type,
		data->'build'->>'versionId' AS build_version_id,
		data->'build'->>'projectVersionId' AS build_project_version_id,
		ip,
		created,
		EXTRACT(DAY FROM created)::smallint AS day,
		DATE(created) AS date_only
	FROM requests
	WHERE 
		data IS NOT NULL 
		AND status = 200
		AND path NOT LIKE '%tracking=nostats%'
)
SELECT
	request_type,
	search_type,
	search_version,
	build_type,
	build_version_id,
	build_project_version_id,
	date_only,
	day,
	COUNT(*)::BIGINT AS total_requests,
	COUNT(DISTINCT ip)::BIGINT AS unique_ips
FROM request_data
GROUP BY 
	request_type,
	search_type,
	search_version,
	build_type,
	build_version_id,
	build_project_version_id,
	date_only,
	day;--> statement-breakpoint

CREATE INDEX idx_mv_daily_request_type ON mv_requests_stats_daily(request_type);--> statement-breakpoint
CREATE INDEX idx_mv_daily_search_type ON mv_requests_stats_daily(search_type);--> statement-breakpoint
CREATE INDEX idx_mv_daily_search_version ON mv_requests_stats_daily(search_version);--> statement-breakpoint
CREATE INDEX idx_mv_daily_build_type ON mv_requests_stats_daily(build_type);--> statement-breakpoint
CREATE INDEX idx_mv_daily_build_version_id ON mv_requests_stats_daily(build_version_id);--> statement-breakpoint
CREATE INDEX idx_mv_daily_build_project_version_id ON mv_requests_stats_daily(build_project_version_id);--> statement-breakpoint
CREATE INDEX idx_mv_daily_date ON mv_requests_stats_daily(date_only);--> statement-breakpoint
CREATE INDEX idx_mv_daily_day ON mv_requests_stats_daily(day);--> statement-breakpoint

CREATE INDEX idx_mv_daily_request_date ON mv_requests_stats_daily(request_type, date_only);--> statement-breakpoint
CREATE INDEX idx_mv_daily_search_version_date ON mv_requests_stats_daily(search_version, date_only);--> statement-breakpoint
CREATE INDEX idx_mv_daily_build_version_date ON mv_requests_stats_daily(build_version_id, date_only);--> statement-breakpoint

DROP INDEX IF EXISTS idx_requests_builds_type;--> statement-breakpoint
DROP INDEX IF EXISTS idx_requests_search_type;--> statement-breakpoint
DROP INDEX IF EXISTS idx_requests_search_type_version;--> statement-breakpoint
DROP INDEX IF EXISTS idx_requests_search_version;--> statement-breakpoint
DROP INDEX IF EXISTS idx_requests_lookup_type;--> statement-breakpoint
DROP INDEX IF EXISTS idx_requests_lookup_versionid;--> statement-breakpoint
DROP INDEX IF EXISTS idx_requests_lookup_id;