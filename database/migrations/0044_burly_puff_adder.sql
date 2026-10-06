ALTER TABLE "builds" ADD COLUMN "sort" serial NOT NULL;--> statement-breakpoint
UPDATE "builds" SET "sort" = "id";--> statement-breakpoint
SELECT setval(pg_get_serial_sequence('builds', 'sort'), (SELECT max("sort") FROM "builds"));--> statement-breakpoint
CREATE INDEX "builds_version_type_sort_idx" ON "builds" USING btree ("version_id","type","sort");--> statement-breakpoint
CREATE INDEX "builds_project_version_type_sort_idx" ON "builds" USING btree ("project_version_id","type","sort");