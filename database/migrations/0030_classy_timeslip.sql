CREATE TABLE "counts" (
	"key" varchar(255) PRIMARY KEY NOT NULL,
	"value" bigint DEFAULT 0 NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX "counts_key_idx" ON "counts" USING btree ("key");