CREATE EXTENSION IF NOT EXISTS pgcrypto;--> statement-breakpoint
DELETE FROM "user_sessions";--> statement-breakpoint
DROP INDEX "userSessions_session_idx";--> statement-breakpoint
ALTER TABLE "organization_keys" ALTER COLUMN "key" SET DATA TYPE text;--> statement-breakpoint
ALTER TABLE "organization_keys" ADD COLUMN "key_id" char(16);--> statement-breakpoint
UPDATE "organization_keys" SET "key_id" = substring("key" from 1 for 16), "key" = crypt("key", gen_salt('bf', 12));--> statement-breakpoint
ALTER TABLE "organization_keys" ALTER COLUMN "key_id" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "user_sessions" ADD COLUMN "key_id" char(16) NOT NULL;--> statement-breakpoint
ALTER TABLE "user_sessions" ADD COLUMN "key" text NOT NULL;--> statement-breakpoint
CREATE INDEX "organizationKeys_key_id_idx" ON "organization_keys" USING btree ("key_id");--> statement-breakpoint
CREATE UNIQUE INDEX "userSessions_key_idx" ON "user_sessions" USING btree ("key");--> statement-breakpoint
CREATE INDEX "userSessions_key_id_idx" ON "user_sessions" USING btree ("key_id");--> statement-breakpoint
ALTER TABLE "user_sessions" DROP COLUMN "session";
