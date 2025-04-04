ALTER TABLE "organization_subusers" ADD COLUMN "pending" boolean DEFAULT true NOT NULL;--> statement-breakpoint
ALTER TABLE "organization_subusers" ADD COLUMN "created" timestamp DEFAULT now() NOT NULL;--> statement-breakpoint
CREATE INDEX "organizationSubusers_userId_pending_idx" ON "organization_subusers" USING btree ("user_id","pending");