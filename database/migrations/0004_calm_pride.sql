DROP INDEX IF EXISTS "builds_type_version_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_type_project_version_idx";--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_version_type_idx" ON "builds" USING btree ("version_id","type");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_project_version_type_idx" ON "builds" USING btree ("project_version_id","type");