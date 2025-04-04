CREATE TABLE IF NOT EXISTS "user_sessions" (
	"id" serial PRIMARY KEY NOT NULL,
	"user_id" integer NOT NULL,
	"session" char(64) NOT NULL,
	"last_used" timestamp DEFAULT now() NOT NULL,
	"created" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "user_sessions" ADD CONSTRAINT "user_sessions_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "userSessions_user_idx" ON "user_sessions" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "userSessions_session_idx" ON "user_sessions" USING btree ("session");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "users_github_id_idx" ON "users" USING btree ("github_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "users_login_idx" ON "users" USING btree ("login");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "users_email_idx" ON "users" USING btree ("email");