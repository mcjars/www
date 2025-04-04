CREATE TABLE IF NOT EXISTS "organization_subusers" (
	"organization_id" integer NOT NULL,
	"user_id" integer NOT NULL,
	CONSTRAINT "organizationSubusers_pk" PRIMARY KEY("organization_id","user_id")
);
--> statement-breakpoint
DROP INDEX IF EXISTS "userSessions_session_idx";--> statement-breakpoint
ALTER TABLE "organizations" ADD COLUMN "owner_id" integer DEFAULT 1 NOT NULL;--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "organization_subusers" ADD CONSTRAINT "organization_subusers_organization_id_organizations_id_fk" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "organization_subusers" ADD CONSTRAINT "organization_subusers_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "organizationSubusers_organization_idx" ON "organization_subusers" USING btree ("organization_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "organizationSubusers_user_idx" ON "organization_subusers" USING btree ("user_id");--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "organizations" ADD CONSTRAINT "organizations_owner_id_users_id_fk" FOREIGN KEY ("owner_id") REFERENCES "public"."users"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
CREATE UNIQUE INDEX IF NOT EXISTS "userSessions_session_idx" ON "user_sessions" USING btree ("session");