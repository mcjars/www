CREATE TABLE "ch_request_stats" (
	"request_type" text,
	"search_type" text,
	"search_version" text,
	"build_type" text,
	"build_version_id" text,
	"build_project_version_id" text,
	"total_requests" integer NOT NULL,
	"unique_ips" integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE "ch_request_stats_daily" (
	"request_type" text,
	"search_type" text,
	"search_version" text,
	"build_type" text,
	"build_version_id" text,
	"build_project_version_id" text,
	"date_only" date NOT NULL,
	"day" smallint NOT NULL,
	"total_requests" integer NOT NULL,
	"unique_ips" integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX "chRequestStats_request_type_idx" ON "ch_request_stats" USING btree ("request_type");--> statement-breakpoint
CREATE INDEX "chRequestStats_search_type_idx" ON "ch_request_stats" USING btree ("search_type");--> statement-breakpoint
CREATE INDEX "chRequestStats_build_type_idx" ON "ch_request_stats" USING btree ("build_type");--> statement-breakpoint
CREATE INDEX "chRequestStats_build_version_id_idx" ON "ch_request_stats" USING btree ("build_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStats_build_project_version_id_idx" ON "ch_request_stats" USING btree ("build_project_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_request_type_idx" ON "ch_request_stats_daily" USING btree ("request_type");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_search_type_idx" ON "ch_request_stats_daily" USING btree ("search_type");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_build_type_idx" ON "ch_request_stats_daily" USING btree ("build_type");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_build_version_id_idx" ON "ch_request_stats_daily" USING btree ("build_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_build_project_version_id_idx" ON "ch_request_stats_daily" USING btree ("build_project_version_id");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_date_only_idx" ON "ch_request_stats_daily" USING brin ("date_only");--> statement-breakpoint
CREATE INDEX "chRequestStatsDaily_day_idx" ON "ch_request_stats_daily" USING btree ("day");