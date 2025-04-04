DROP INDEX "organizationKeys_key_idx";--> statement-breakpoint
DROP INDEX "requests_data_idx";--> statement-breakpoint
DROP INDEX "requests_created_idx";--> statement-breakpoint
ALTER TABLE "user_sessions" ADD COLUMN "ip" "inet" NOT NULL;--> statement-breakpoint
ALTER TABLE "user_sessions" ADD COLUMN "user_agent" varchar(255) NOT NULL;--> statement-breakpoint
CREATE UNIQUE INDEX "organizationKeys_organization_name_idx" ON "organization_keys" USING btree ("organization_id","name");--> statement-breakpoint
CREATE INDEX "requests_data_idx" ON "requests" USING gin ("data") WHERE "requests"."data" is not null;--> statement-breakpoint
CREATE INDEX "requests_created_idx" ON "requests" USING brin ("created");--> statement-breakpoint
ALTER TABLE "organization_keys" ADD CONSTRAINT "organization_keys_key_unique" UNIQUE("key");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS idx_requests_lookup_type 
ON requests ((data->'build'->>'type'), (data->>'type'))
WHERE data->>'type' = 'lookup';--> statement-breakpoint
CREATE INDEX IF NOT EXISTS idx_requests_lookup_versionid 
ON requests ((data->'build'->>'versionId'), (data->>'type'))
WHERE data->>'type' = 'lookup';--> statement-breakpoint
CREATE INDEX IF NOT EXISTS idx_requests_lookup_id 
ON requests ((data->'build'->>'id'), (data->>'type'))
WHERE data->>'type' = 'lookup' AND data->'build'->>'type' = '$1';