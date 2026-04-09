ALTER TABLE "builds" ADD COLUMN "uuid" uuid DEFAULT gen_random_uuid() NOT NULL;--> statement-breakpoint
ALTER TABLE "config_values" ADD COLUMN "uuid" uuid DEFAULT gen_random_uuid() NOT NULL;--> statement-breakpoint
ALTER TABLE "config_values" ADD COLUMN "parsed" jsonb DEFAULT '{}'::jsonb NOT NULL;--> statement-breakpoint
ALTER TABLE "configs" ADD COLUMN "uuid" uuid DEFAULT gen_random_uuid() NOT NULL;--> statement-breakpoint
CREATE UNIQUE INDEX "builds_uuid_idx" ON "builds" USING btree ("uuid");--> statement-breakpoint
CREATE UNIQUE INDEX "configValues_uuid_idx" ON "config_values" USING btree ("uuid");--> statement-breakpoint
CREATE UNIQUE INDEX "configs_uuid_idx" ON "configs" USING btree ("uuid");