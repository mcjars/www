ALTER TABLE "organizations" ALTER COLUMN "icon" SET DEFAULT 'https://s3.mcjars.app/organization-icons/default.webp';--> statement-breakpoint
ALTER TABLE "organizations" ALTER COLUMN "icon" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "organizations" ADD COLUMN "verified" boolean DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE "organizations" ADD COLUMN "public" boolean DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE "users" ADD COLUMN "admin" boolean DEFAULT false NOT NULL;