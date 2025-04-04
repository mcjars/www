ALTER TABLE "organizations" ALTER COLUMN "icon" DROP NOT NULL;--> statement-breakpoint
ALTER TABLE "organizations" ADD COLUMN "created" timestamp DEFAULT now() NOT NULL;