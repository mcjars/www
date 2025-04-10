ALTER TABLE build_hashes
	ADD COLUMN sha1_bytea BYTEA,
	ADD COLUMN sha224_bytea BYTEA,
	ADD COLUMN sha256_bytea BYTEA,
	ADD COLUMN sha384_bytea BYTEA,
	ADD COLUMN sha512_bytea BYTEA,
	ADD COLUMN md5_bytea BYTEA;--> statement-breakpoint
UPDATE build_hashes SET
  sha1_bytea = decode(sha1, 'hex'),
  sha224_bytea = decode(sha224, 'hex'),
  sha256_bytea = decode(sha256, 'hex'),
  sha384_bytea = decode(sha384, 'hex'),
  sha512_bytea = decode(sha512, 'hex'),
  md5_bytea = decode(md5, 'hex');--> statement-breakpoint

DROP INDEX "buildHashes_sha1_idx";--> statement-breakpoint
DROP INDEX "buildHashes_sha224_idx";--> statement-breakpoint
DROP INDEX "buildHashes_sha256_idx";--> statement-breakpoint
DROP INDEX "buildHashes_sha384_idx";--> statement-breakpoint
DROP INDEX "buildHashes_sha512_idx";--> statement-breakpoint
DROP INDEX "buildHashes_md5_idx";--> statement-breakpoint

ALTER TABLE build_hashes
	DROP COLUMN sha1,
	DROP COLUMN sha224,
	DROP COLUMN sha256,
	DROP COLUMN sha384,
	DROP COLUMN sha512,
	DROP COLUMN md5;--> statement-breakpoint
ALTER TABLE build_hashes
RENAME COLUMN sha1_bytea TO sha1;--> statement-breakpoint
ALTER TABLE build_hashes
RENAME COLUMN sha224_bytea TO sha224;--> statement-breakpoint
ALTER TABLE build_hashes
RENAME COLUMN sha256_bytea TO sha256;--> statement-breakpoint
ALTER TABLE build_hashes
RENAME COLUMN sha384_bytea TO sha384;--> statement-breakpoint
ALTER TABLE build_hashes
RENAME COLUMN sha512_bytea TO sha512;--> statement-breakpoint
ALTER TABLE build_hashes
RENAME COLUMN md5_bytea TO md5;--> statement-breakpoint

CREATE INDEX "buildHashes_sha1_idx" ON "build_hashes" USING btree ("sha1") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "buildHashes_sha224_idx" ON "build_hashes" USING btree ("sha224") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "buildHashes_sha256_idx" ON "build_hashes" USING btree ("sha256") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "buildHashes_sha384_idx" ON "build_hashes" USING btree ("sha384") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "buildHashes_sha512_idx" ON "build_hashes" USING btree ("sha512") WITH (fillfactor=100);--> statement-breakpoint
CREATE INDEX "buildHashes_md5_idx" ON "build_hashes" USING btree ("md5") WITH (fillfactor=100);