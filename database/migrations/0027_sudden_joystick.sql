ALTER TYPE "public"."server_type" ADD VALUE 'LEAF';--> statement-breakpoint
ALTER TABLE "webhooks" ALTER COLUMN "types" SET DEFAULT '["VANILLA","PAPER","PUFFERFISH","SPIGOT","FOLIA","PURPUR","WATERFALL","VELOCITY","FABRIC","BUNGEECORD","QUILT","FORGE","NEOFORGE","MOHIST","ARCLIGHT","SPONGE","LEAVES","CANVAS","ASPAPER","LEGACY_FABRIC","LOOHP_LIMBO","NANOLIMBO","DIVINEMC","MAGMA","LEAF"]'::jsonb;