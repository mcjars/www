CREATE EXTENSION IF NOT EXISTS pg_trgm;--> statement-breakpoint
CREATE TYPE "public"."format" AS ENUM('YAML', 'CONF', 'TOML', 'PROPERTIES');--> statement-breakpoint
CREATE TYPE "public"."method" AS ENUM('GET', 'POST', 'PUT', 'DELETE', 'PATCH');--> statement-breakpoint
CREATE TYPE "public"."server_type" AS ENUM('VANILLA', 'PAPER', 'PUFFERFISH', 'SPIGOT', 'FOLIA', 'PURPUR', 'WATERFALL', 'VELOCITY', 'FABRIC', 'BUNGEECORD', 'QUILT', 'FORGE', 'NEOFORGE', 'MOHIST', 'ARCLIGHT', 'SPONGE', 'LEAVES', 'CANVAS');--> statement-breakpoint
CREATE TYPE "public"."version_type" AS ENUM('RELEASE', 'SNAPSHOT');--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "build_configs" (
	"build_id" integer NOT NULL,
	"config_id" integer NOT NULL,
	"config_value_id" integer NOT NULL,
	CONSTRAINT "buildConfigs_pk" PRIMARY KEY("build_id","config_id")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "build_hashes" (
	"build_id" integer NOT NULL,
	"primary" boolean NOT NULL,
	"sha1" char(40) NOT NULL,
	"sha224" char(56) NOT NULL,
	"sha256" char(64) NOT NULL,
	"sha384" char(96) NOT NULL,
	"sha512" char(128) NOT NULL,
	"md5" char(32) NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "builds" (
	"id" serial PRIMARY KEY NOT NULL,
	"version_id" varchar(63),
	"project_version_id" varchar(63),
	"type" "server_type" NOT NULL,
	"rehash" boolean DEFAULT false NOT NULL,
	"experimental" boolean DEFAULT false NOT NULL,
	"build_number" integer NOT NULL,
	"jar_url" varchar(255),
	"jar_size" integer,
	"jar_location" varchar(51),
	"zip_url" varchar(255),
	"zip_size" integer,
	"metadata" jsonb NOT NULL,
	"installation" jsonb NOT NULL,
	"changes" jsonb NOT NULL,
	"created" timestamp
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "config_values" (
	"id" serial PRIMARY KEY NOT NULL,
	"config_id" integer NOT NULL,
	"sha1" char(40) NOT NULL,
	"sha224" char(56) NOT NULL,
	"sha256" char(64) NOT NULL,
	"sha384" char(96) NOT NULL,
	"sha512" char(128) NOT NULL,
	"md5" char(32) NOT NULL,
	"value" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "configs" (
	"id" serial PRIMARY KEY NOT NULL,
	"location" varchar(51) NOT NULL,
	"type" "server_type" NOT NULL,
	"format" "format" NOT NULL,
	CONSTRAINT "configs_location_unique" UNIQUE("location")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "minecraft_versions" (
	"id" varchar(63) PRIMARY KEY NOT NULL,
	"type" "version_type" NOT NULL,
	"supported" boolean NOT NULL,
	"java" smallint DEFAULT 21 NOT NULL,
	"created" timestamp NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "organization_keys" (
	"id" serial PRIMARY KEY NOT NULL,
	"organization_id" integer NOT NULL,
	"key" char(64) NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "organizations" (
	"id" serial PRIMARY KEY NOT NULL,
	"name" varchar(255) NOT NULL,
	"icon" varchar(255) NOT NULL,
	"types" jsonb NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "project_versions" (
	"id" varchar(63) NOT NULL,
	"type" "server_type" NOT NULL,
	CONSTRAINT "projectVersions_pk" PRIMARY KEY("type","id")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "requests" (
	"id" char(12) PRIMARY KEY NOT NULL,
	"organization_id" integer,
	"origin" varchar(255),
	"method" "method" NOT NULL,
	"path" varchar(255) NOT NULL,
	"time" integer NOT NULL,
	"status" smallint NOT NULL,
	"body" jsonb,
	"ip" "inet" NOT NULL,
	"continent" char(2),
	"country" char(2),
	"data" jsonb,
	"user_agent" varchar(255) NOT NULL,
	"created" timestamp NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "webhooks" (
	"id" serial PRIMARY KEY NOT NULL,
	"organization_id" integer,
	"label" varchar(255),
	"name" varchar(63) NOT NULL,
	"avatar" varchar(255) NOT NULL,
	"url" varchar(255) NOT NULL,
	"types" jsonb DEFAULT '["VANILLA","PAPER","PUFFERFISH","SPIGOT","FOLIA","PURPUR","WATERFALL","VELOCITY","FABRIC","BUNGEECORD","QUILT","FORGE","NEOFORGE","MOHIST","ARCLIGHT","SPONGE","LEAVES","CANVAS"]'::jsonb NOT NULL,
	"enabled" boolean DEFAULT true NOT NULL,
	"successful" integer DEFAULT 0 NOT NULL,
	"failed" integer DEFAULT 0 NOT NULL
);
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "build_configs" ADD CONSTRAINT "build_configs_build_id_builds_id_fk" FOREIGN KEY ("build_id") REFERENCES "public"."builds"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "build_configs" ADD CONSTRAINT "build_configs_config_id_configs_id_fk" FOREIGN KEY ("config_id") REFERENCES "public"."configs"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "build_configs" ADD CONSTRAINT "build_configs_config_value_id_config_values_id_fk" FOREIGN KEY ("config_value_id") REFERENCES "public"."config_values"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "build_hashes" ADD CONSTRAINT "build_hashes_build_id_builds_id_fk" FOREIGN KEY ("build_id") REFERENCES "public"."builds"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "builds" ADD CONSTRAINT "builds_version_id_minecraft_versions_id_fk" FOREIGN KEY ("version_id") REFERENCES "public"."minecraft_versions"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "builds" ADD CONSTRAINT "builds_project_version_fk" FOREIGN KEY ("type","project_version_id") REFERENCES "public"."project_versions"("type","id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "config_values" ADD CONSTRAINT "config_values_config_id_configs_id_fk" FOREIGN KEY ("config_id") REFERENCES "public"."configs"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "organization_keys" ADD CONSTRAINT "organization_keys_organization_id_organizations_id_fk" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "requests" ADD CONSTRAINT "requests_organization_id_organizations_id_fk" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations"("id") ON DELETE set null ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "webhooks" ADD CONSTRAINT "webhooks_organization_id_organizations_id_fk" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations"("id") ON DELETE restrict ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildConfigs_build_idx" ON "build_configs" USING btree ("build_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildConfigs_config_idx" ON "build_configs" USING btree ("config_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildConfigs_config_value_idx" ON "build_configs" USING btree ("config_value_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_build_idx" ON "build_hashes" USING btree ("build_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_primary_idx" ON "build_hashes" USING btree ("primary");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_sha1_idx" ON "build_hashes" USING btree ("sha1");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_sha224_idx" ON "build_hashes" USING btree ("sha224");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_sha256_idx" ON "build_hashes" USING btree ("sha256");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_sha384_idx" ON "build_hashes" USING btree ("sha384");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_sha512_idx" ON "build_hashes" USING btree ("sha512");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "buildHashes_md5_idx" ON "build_hashes" USING btree ("md5");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_type_idx" ON "builds" USING btree ("type");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_experimental_idx" ON "builds" USING btree ("experimental");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_build_number_idx" ON "builds" USING btree ("build_number");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_jar_url_idx" ON "builds" USING btree ("jar_url");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_jar_size_idx" ON "builds" USING btree ("jar_size");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_jar_location_idx" ON "builds" USING btree ("jar_location");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_zip_url_idx" ON "builds" USING btree ("zip_url");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_zip_size_idx" ON "builds" USING btree ("zip_size");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_created_idx" ON "builds" USING btree ("created");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_type_version_idx" ON "builds" USING btree ("type","version_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_type_project_version_idx" ON "builds" USING btree ("type","project_version_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_version_idx" ON "builds" USING btree ("version_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "builds_project_version_idx" ON "builds" USING btree ("project_version_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "configValues_config_idx" ON "config_values" USING btree ("config_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "configValues_value_idx" ON "config_values" USING btree ("value");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "configValues_value_trgm_idx" ON "public"."config_values" USING GIN("value" gin_trgm_ops);--> statement-breakpoint
CREATE UNIQUE INDEX IF NOT EXISTS "configValues_unique_config_value_idx" ON "config_values" USING btree ("config_id","sha1","sha224","sha256","sha384","sha512","md5");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "configs_type_idx" ON "configs" USING btree ("type");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "configs_format_idx" ON "configs" USING btree ("format");--> statement-breakpoint
CREATE UNIQUE INDEX IF NOT EXISTS "configs_unique_location_idx" ON "configs" USING btree ("location");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "minecraftVersions_type_idx" ON "minecraft_versions" USING btree ("type");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "minecraftVersions_java_idx" ON "minecraft_versions" USING btree ("java");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "organizationKeys_organization_idx" ON "organization_keys" USING btree ("organization_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "organizationKeys_key_idx" ON "organization_keys" USING btree ("key");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "organizations_name_idx" ON "organizations" USING btree ("name");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "projectVersions_type_idx" ON "project_versions" USING btree ("type");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_organization_idx" ON "requests" USING btree ("organization_id") WHERE "requests"."organization_id" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_ip_idx" ON "requests" USING btree ("ip");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_path_idx" ON "requests" USING btree ("path");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_continent_idx" ON "requests" USING btree ("continent") WHERE "requests"."continent" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_country_idx" ON "requests" USING btree ("country") WHERE "requests"."country" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_data_idx" ON "requests" USING btree ("data") WHERE "requests"."data" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "requests_created_idx" ON "requests" USING btree ("created");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "webhooks_organization_idx" ON "webhooks" USING btree ("organization_id") WHERE "webhooks"."organization_id" is not null;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "webhooks_enabled_idx" ON "webhooks" USING btree ("enabled");