ALTER TYPE "public"."server_type" ADD VALUE 'LOOHP_LIMBO';--> statement-breakpoint
ALTER TYPE "public"."server_type" ADD VALUE 'NANOLIMBO';--> statement-breakpoint
ALTER TABLE "configs" DROP CONSTRAINT "configs_location_unique";--> statement-breakpoint
DROP INDEX "builds_created_idx";--> statement-breakpoint
ALTER TABLE "organizations" ALTER COLUMN "owner_id" DROP DEFAULT;--> statement-breakpoint
ALTER TABLE "organizations" ALTER COLUMN "types" SET DEFAULT '[]'::jsonb;--> statement-breakpoint
ALTER TABLE "webhooks" ALTER COLUMN "types" SET DEFAULT '["VANILLA","PAPER","PUFFERFISH","SPIGOT","FOLIA","PURPUR","WATERFALL","VELOCITY","FABRIC","BUNGEECORD","QUILT","FORGE","NEOFORGE","MOHIST","ARCLIGHT","SPONGE","LEAVES","CANVAS","ASPAPER","LEGACY_FABRIC","LOOHP_LIMBO","NANOLIMBO"]'::jsonb;--> statement-breakpoint
CREATE INDEX "builds_created_idx" ON "builds" USING btree ("created");