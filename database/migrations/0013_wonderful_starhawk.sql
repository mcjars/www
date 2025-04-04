ALTER TABLE "users" DROP CONSTRAINT "users_github_id_unique";--> statement-breakpoint
DROP INDEX IF EXISTS "configs_unique_location_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_jar_url_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_jar_size_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_zip_url_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "builds_zip_size_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "users_github_id_idx";--> statement-breakpoint
DROP INDEX IF EXISTS "users_login_idx";--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_changes_idx" ON "builds" USING btree ("changes") WHERE jsonb_array_length(changes) > 0 AND jsonb_array_length(changes) < 10;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_jar_url_idx" ON "builds" USING btree ("jar_url") WHERE "builds"."jar_url" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_jar_size_idx" ON "builds" USING btree ("jar_size") WHERE "builds"."jar_size" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_zip_url_idx" ON "builds" USING btree ("zip_url") WHERE "builds"."zip_url" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_zip_size_idx" ON "builds" USING btree ("zip_size") WHERE "builds"."zip_size" is not null;--> statement-breakpoint
CREATE UNIQUE INDEX IF NOT EXISTS "users_github_id_idx" ON "users" USING btree ("github_id");--> statement-breakpoint
CREATE UNIQUE INDEX IF NOT EXISTS "users_login_idx" ON "users" USING btree ("login");