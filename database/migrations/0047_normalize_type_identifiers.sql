UPDATE "webhooks" SET "types" = replace(replace("types"::text, '"LEGACYFABRIC"', '"LEGACY_FABRIC"'), '"LOOHPLIMBO"', '"LOOHP_LIMBO"')::jsonb WHERE "types"::text ~ '"(LEGACYFABRIC|LOOHPLIMBO)"';--> statement-breakpoint
UPDATE "organizations" SET "types" = replace(replace("types"::text, '"LEGACYFABRIC"', '"LEGACY_FABRIC"'), '"LOOHPLIMBO"', '"LOOHP_LIMBO"')::jsonb WHERE "types"::text ~ '"(LEGACYFABRIC|LOOHPLIMBO)"';--> statement-breakpoint
WITH moved AS (
	DELETE FROM "ch_request_stats"
	WHERE "search_type" IN ('LEGACYFABRIC', 'LOOHPLIMBO') OR "build_type" IN ('LEGACYFABRIC', 'LOOHPLIMBO')
	RETURNING *
)
INSERT INTO "ch_request_stats" ("request_type", "search_type", "search_version", "build_type", "build_version_id", "build_project_version_id", "total_requests", "unique_ips")
SELECT
	"request_type",
	CASE "search_type" WHEN 'LEGACYFABRIC' THEN 'LEGACY_FABRIC' WHEN 'LOOHPLIMBO' THEN 'LOOHP_LIMBO' ELSE "search_type" END,
	"search_version",
	CASE "build_type" WHEN 'LEGACYFABRIC' THEN 'LEGACY_FABRIC' WHEN 'LOOHPLIMBO' THEN 'LOOHP_LIMBO' ELSE "build_type" END,
	"build_version_id",
	"build_project_version_id",
	SUM("total_requests"),
	SUM("unique_ips")
FROM moved
GROUP BY 1, 2, 3, 4, 5, 6
ON CONFLICT ("request_type", "search_type", "search_version", "build_type", "build_version_id", "build_project_version_id")
DO UPDATE SET
	"total_requests" = "ch_request_stats"."total_requests" + EXCLUDED."total_requests",
	"unique_ips" = "ch_request_stats"."unique_ips" + EXCLUDED."unique_ips";--> statement-breakpoint
WITH moved AS (
	DELETE FROM "ch_request_stats_daily"
	WHERE "search_type" IN ('LEGACYFABRIC', 'LOOHPLIMBO') OR "build_type" IN ('LEGACYFABRIC', 'LOOHPLIMBO')
	RETURNING *
)
INSERT INTO "ch_request_stats_daily" ("request_type", "search_type", "search_version", "build_type", "build_version_id", "build_project_version_id", "date_only", "day", "total_requests", "unique_ips")
SELECT
	"request_type",
	CASE "search_type" WHEN 'LEGACYFABRIC' THEN 'LEGACY_FABRIC' WHEN 'LOOHPLIMBO' THEN 'LOOHP_LIMBO' ELSE "search_type" END,
	"search_version",
	CASE "build_type" WHEN 'LEGACYFABRIC' THEN 'LEGACY_FABRIC' WHEN 'LOOHPLIMBO' THEN 'LOOHP_LIMBO' ELSE "build_type" END,
	"build_version_id",
	"build_project_version_id",
	"date_only",
	"day",
	SUM("total_requests"),
	SUM("unique_ips")
FROM moved
GROUP BY 1, 2, 3, 4, 5, 6, 7, 8
ON CONFLICT ("request_type", "search_type", "search_version", "build_type", "build_version_id", "build_project_version_id", "date_only")
DO UPDATE SET
	"total_requests" = "ch_request_stats_daily"."total_requests" + EXCLUDED."total_requests",
	"unique_ips" = "ch_request_stats_daily"."unique_ips" + EXCLUDED."unique_ips";
