DROP INDEX "chRequestStats_request_type_idx";--> statement-breakpoint
DROP INDEX "chRequestStats_search_type_idx";--> statement-breakpoint
DROP INDEX "chRequestStats_build_version_id_idx";--> statement-breakpoint
DROP INDEX "chRequestStats_build_project_version_id_idx";--> statement-breakpoint
DROP INDEX "chRequestStatsDaily_request_type_idx";--> statement-breakpoint
DROP INDEX "chRequestStatsDaily_search_type_idx";--> statement-breakpoint
DROP INDEX "chRequestStatsDaily_build_type_idx";--> statement-breakpoint
DROP INDEX "chRequestStatsDaily_build_version_id_idx";--> statement-breakpoint
DROP INDEX "chRequestStatsDaily_build_project_version_id_idx";--> statement-breakpoint
DROP INDEX "chRequestStatsDaily_date_only_idx";--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "request_type" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "search_type" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "search_version" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "build_type" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "build_version_id" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "build_project_version_id" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "total_requests" SET DATA TYPE bigint;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ALTER COLUMN "unique_ips" SET DATA TYPE bigint;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "request_type" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "search_type" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "search_version" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "build_type" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "build_version_id" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "build_project_version_id" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "total_requests" SET DATA TYPE bigint;--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ALTER COLUMN "unique_ips" SET DATA TYPE bigint;--> statement-breakpoint
ALTER TABLE "ch_request_stats" ADD CONSTRAINT "ch_request_stats_request_type_search_type_search_version_build_type_build_version_id_build_project_version_id_pk" PRIMARY KEY("request_type","search_type","search_version","build_type","build_version_id","build_project_version_id");--> statement-breakpoint
ALTER TABLE "ch_request_stats_daily" ADD CONSTRAINT "ch_request_stats_daily_request_type_search_type_search_version_build_type_build_version_id_build_project_version_id_date_only_pk" PRIMARY KEY("request_type","search_type","search_version","build_type","build_version_id","build_project_version_id","date_only");--> statement-breakpoint
CREATE INDEX "chRequestStats_req_search_type_idx" ON "ch_request_stats" USING btree ("request_type","search_type");--> statement-breakpoint
CREATE INDEX "chRequestStats_req_search_ver_idx" ON "ch_request_stats" USING btree ("request_type","search_version");--> statement-breakpoint
CREATE INDEX "chRequestStats_req_build_ver_idx" ON "ch_request_stats" USING btree ("request_type","build_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStats_search_ver_idx" ON "ch_request_stats" USING btree ("search_version");--> statement-breakpoint
CREATE INDEX "chRequestStats_build_vid_idx" ON "ch_request_stats" USING btree ("build_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStats_build_pvid_idx" ON "ch_request_stats" USING btree ("build_project_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_req_date_idx" ON "ch_request_stats_daily" USING btree ("request_type","date_only");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_search_ver_date_idx" ON "ch_request_stats_daily" USING btree ("search_version","date_only");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_build_ver_date_idx" ON "ch_request_stats_daily" USING btree ("build_version_id","date_only");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_date_idx" ON "ch_request_stats_daily" USING btree ("date_only");