DROP INDEX IF EXISTS "builds_jar_location_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_project_version_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_created_idx";--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_created_idx" ON "builds" USING btree ("created") WHERE "builds"."created" is not null;