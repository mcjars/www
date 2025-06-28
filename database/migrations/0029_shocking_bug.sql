ALTER TABLE "builds" ADD COLUMN "name" varchar(255) DEFAULT '' NOT NULL;--> statement-breakpoint
CREATE INDEX "builds_name_idx" ON "builds" USING btree ("name");