CREATE TABLE "ch_file_stats" (
	"root" text NOT NULL,
	"path" text NOT NULL,
	"kind" text NOT NULL,
	"extension" text NOT NULL,
	"total_requests" bigint NOT NULL,
	"unique_ips" bigint NOT NULL,
	"total_bytes" bigint NOT NULL,
	CONSTRAINT "ch_file_stats_root_path_kind_extension_pk" PRIMARY KEY("root","path","kind","extension")
);
--> statement-breakpoint
CREATE TABLE "ch_file_stats_daily" (
	"root" text NOT NULL,
	"path" text NOT NULL,
	"kind" text NOT NULL,
	"extension" text NOT NULL,
	"date_only" date NOT NULL,
	"day" smallint NOT NULL,
	"total_requests" bigint NOT NULL,
	"unique_ips" bigint NOT NULL,
	"total_bytes" bigint NOT NULL,
	CONSTRAINT "ch_file_stats_daily_root_path_kind_extension_date_only_pk" PRIMARY KEY("root","path","kind","extension","date_only")
);
--> statement-breakpoint
CREATE INDEX "chFileStats_root_idx" ON "ch_file_stats" USING btree ("root");--> statement-breakpoint
CREATE INDEX "chFileStats_kind_idx" ON "ch_file_stats" USING btree ("kind");--> statement-breakpoint
CREATE INDEX "chFileStats_root_kind_idx" ON "ch_file_stats" USING btree ("root","kind");--> statement-breakpoint
CREATE INDEX "chFileStats_extension_idx" ON "ch_file_stats" USING btree ("extension");--> statement-breakpoint
CREATE INDEX "chFileStatsDaily_root_date_idx" ON "ch_file_stats_daily" USING btree ("root","date_only");--> statement-breakpoint
CREATE INDEX "chFileStatsDaily_kind_date_idx" ON "ch_file_stats_daily" USING btree ("kind","date_only");--> statement-breakpoint
CREATE INDEX "chFileStatsDaily_date_idx" ON "ch_file_stats_daily" USING btree ("date_only");--> statement-breakpoint
CREATE INDEX "chFileStatsDaily_day_idx" ON "ch_file_stats_daily" USING btree ("day");