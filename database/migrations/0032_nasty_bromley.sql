CREATE TABLE "files" (
	"path" varchar(255)[],
	"size" integer NOT NULL,
	"sha1" "bytea" NOT NULL,
	"sha224" "bytea" NOT NULL,
	"sha256" "bytea" NOT NULL,
	"sha384" "bytea" NOT NULL,
	"sha512" "bytea" NOT NULL,
	"md5" "bytea" NOT NULL,
	CONSTRAINT "files_pk" PRIMARY KEY("path")
);
--> statement-breakpoint
CREATE INDEX "files_sha1_idx" ON "files" USING btree ("sha1") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "files_sha224_idx" ON "files" USING btree ("sha224") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "files_sha256_idx" ON "files" USING btree ("sha256") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "files_sha384_idx" ON "files" USING btree ("sha384") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "files_sha512_idx" ON "files" USING btree ("sha512") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "files_md5_idx" ON "files" USING btree ("md5") WITH (fillfactor=100);