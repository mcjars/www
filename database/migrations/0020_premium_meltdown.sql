DROP INDEX "organizations_name_idx";--> statement-breakpoint
CREATE UNIQUE INDEX "organizations_name_idx" ON "organizations" USING btree ("name");