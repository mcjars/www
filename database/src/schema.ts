import { isNotNull, relations, sql } from "drizzle-orm"
import { foreignKey, index, integer, primaryKey, pgTable, varchar, uniqueIndex, pgEnum, serial, jsonb, char, boolean, smallint, timestamp, inet, text, customType } from "drizzle-orm/pg-core"

export const bytea = customType<{ data: string; notNull: false; default: false }>({
  dataType() {
    return 'bytea'
  },

  toDriver(val: string) {
    let newVal = val;
    if (val.startsWith("0x")) {
      newVal = val.slice(2);
    }

    return Buffer.from(newVal, "hex");
  },

  fromDriver(val: unknown) {
    return (val as Buffer).toString("hex");
  }
})

export const types = [
	'VANILLA',
	'PAPER',
	'PUFFERFISH',
	'SPIGOT',
	'FOLIA',
	'PURPUR',
	'WATERFALL',
	'VELOCITY',
	'FABRIC',
	'BUNGEECORD',
	'QUILT',
	'FORGE',
	'NEOFORGE',
	'MOHIST',
	'ARCLIGHT',
	'SPONGE',
	'LEAVES',
	'CANVAS',
	'ASPAPER',
	'LEGACY_FABRIC',
	'LOOHP_LIMBO',
	'NANOLIMBO',
	'DIVINEMC'
] as const

export const formats = [
	'YAML',
	'CONF',
	'TOML',
	'PROPERTIES'
] as const

export const typesEnum = pgEnum('server_type', types),
	versionTypeEnum = pgEnum('version_type', ['RELEASE', 'SNAPSHOT']),
	formatsEnum = pgEnum('format', formats),
	methodEnum = pgEnum('method', ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])

export type ServerType = typeof types[number]
export type Format = typeof formats[number]
export type Method = typeof methodEnum['enumValues'][number]

export type InstallStep = {
	type: 'download'

	file: string
	url: string
	size: number
} | {
	type: 'unzip'

	file: string
	location: string
} | {
	type: 'remove'

	location: string
}

export const organizations = pgTable('organizations', {
	id: serial('id').primaryKey().notNull(),
	ownerId: integer('owner_id').notNull().references(() => users.id, { onDelete: 'cascade' }),

	verified: boolean('verified').default(false).notNull(),
	public: boolean('public').default(false).notNull(),

	name: varchar('name', { length: 255 }).notNull(),
	icon: varchar('icon', { length: 255 }).default('https://s3.mcjars.app/organization-icons/default.webp').notNull(),
	types: jsonb('types').default([]).$type<ServerType[]>().notNull(),

	created: timestamp('created').default(sql`now()`).notNull()
}, (organizations) => [
	uniqueIndex('organizations_name_idx').on(organizations.name)
])

export const organizationKeys = pgTable('organization_keys', {
	id: serial('id').primaryKey().notNull(),
	organizationId: integer('organization_id').notNull().references(() => organizations.id, { onDelete: 'cascade' }),

	name: varchar('name', { length: 255 }).notNull().default('Key'),
	key: char('key', { length: 64 }).notNull().unique(),

	created: timestamp('created').default(sql`now()`).notNull()
}, (organizationKeys) => [
	uniqueIndex('organizationKeys_organization_name_idx').on(organizationKeys.organizationId, organizationKeys.name),
	index('organizationKeys_organization_idx').on(organizationKeys.organizationId)
])

export const organizationSubusers = pgTable('organization_subusers', {
	organizationId: integer('organization_id').notNull().references(() => organizations.id, { onDelete: 'cascade' }),
	userId: integer('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
	pending: boolean('pending').default(true).notNull(),
	created: timestamp('created').default(sql`now()`).notNull()
}, (organizationSubusers) => [
	primaryKey({ name: 'organizationSubusers_pk', columns: [organizationSubusers.organizationId, organizationSubusers.userId] }),
	index('organizationSubusers_organization_idx').on(organizationSubusers.organizationId),
	index('organizationSubusers_user_idx').on(organizationSubusers.userId),
	index('organizationSubusers_userId_pending_idx').on(organizationSubusers.userId, organizationSubusers.pending)
])

export const webhooks = pgTable('webhooks', {
	id: serial('id').primaryKey().notNull(),
	organizationId: integer('organization_id').references(() => organizations.id, { onDelete: 'restrict' }),

	label: varchar('label', { length: 255 }),
	url: varchar('url', { length: 255 }).notNull(),
	types: jsonb('types').$type<ServerType[]>().default([...types]).notNull(),
	enabled: boolean('enabled').default(true).notNull(),
	successful: integer('successful').default(0).notNull(),
	failed: integer('failed').default(0).notNull()
}, (webhooks) => [
	index('webhooks_organization_idx').on(webhooks.organizationId).where(isNotNull(webhooks.organizationId)),
	index('webhooks_enabled_idx').on(webhooks.enabled)
])

export const users = pgTable('users', {
	id: serial('id').primaryKey().notNull(),
	githubId: integer('github_id').notNull(),

	admin: boolean('admin').default(false).notNull(),

	name: varchar('name', { length: 255 }),
	email: varchar('email', { length: 255 }).notNull(),
	login: varchar('login', { length: 255 }).notNull(),

	lastLogin: timestamp('last_login').default(sql`now()`).notNull(),
	created: timestamp('created').default(sql`now()`).notNull()
}, (users) => [
	uniqueIndex('users_github_id_idx').on(users.githubId),
	uniqueIndex('users_login_idx').on(users.login),
	index('users_email_idx').on(users.email)
])

export const userSessions = pgTable('user_sessions', {
	id: serial('id').primaryKey().notNull(),
	userId: integer('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),

	session: char('session', { length: 64 }).notNull(),

	ip: inet('ip').notNull(),
	userAgent: varchar('user_agent', { length: 255 }).notNull(),

	lastUsed: timestamp('last_used').default(sql`now()`).notNull(),
	created: timestamp('created').default(sql`now()`).notNull()
}, (userSessions) => [
	index('userSessions_user_idx').on(userSessions.userId),
	uniqueIndex('userSessions_session_idx').on(userSessions.session)
])

export const requests = pgTable('requests', {
	id: char('id', { length: 12 }).primaryKey().notNull(),
	organizationId: integer('organization_id').references(() => organizations.id, { onDelete: 'set null' }),

	origin: varchar('origin', { length: 255 }),
	method: methodEnum('method').notNull(),
	path: varchar('path', { length: 255 }).notNull(),
	time: integer('time').notNull(),
	status: smallint('status').notNull(),
	body: jsonb('body'),
	ip: inet('ip').notNull(),
	continent: char('continent', { length: 2 }),
	country: char('country', { length: 2 }),
	data: jsonb('data'),
	userAgent: varchar('user_agent', { length: 255 }).notNull(),
	created: timestamp('created').notNull()
}, (requests) => [
	index('requests_organization_idx').on(requests.organizationId).where(isNotNull(requests.organizationId)),
	index('requests_status_idx').on(requests.status).where(sql`status = 200`),
	index('requests_ip_idx').on(requests.ip),
	index('requests_continent_idx').on(requests.continent).where(isNotNull(requests.continent)),
	index('requests_country_idx').on(requests.country).where(isNotNull(requests.country)),
	index('requests_created_idx').using('brin', requests.created)
])

export const minecraftVersions = pgTable('minecraft_versions', {
	id: varchar('id', { length: 63 }).primaryKey().notNull(),

	type: versionTypeEnum('type').notNull(),
	supported: boolean('supported').notNull(),
	java: smallint('java').default(21).notNull(),

	created: timestamp('created').notNull()
}, (minecraftVersions) => [
	index('minecraftVersions_type_idx').on(minecraftVersions.type),
	index('minecraftVersions_java_idx').on(minecraftVersions.java)
])

export const projectVersions = pgTable('project_versions', {
	id: varchar('id', { length: 63 }).notNull(),
	type: typesEnum('type').notNull()
}, (projectVersions) => [
	primaryKey({ name: 'projectVersions_pk', columns: [projectVersions.type, projectVersions.id] }),

	index('projectVersions_type_idx').on(projectVersions.type)
])

export const minecraftVersionsRelations = relations(minecraftVersions, ({ many }) => ({
	builds: many(builds, { relationName: 'versions' })
}))

export const projectVersionsRelations = relations(projectVersions, ({ many }) => ({
	builds: many(builds, { relationName: 'project_versions' })
}))

export const builds = pgTable('builds', {
	id: serial('id').primaryKey().notNull(),
	versionId: varchar('version_id', { length: 63 }).references(() => minecraftVersions.id, { onDelete: 'cascade' }),
	projectVersionId: varchar('project_version_id', { length: 63 }),

	type: typesEnum('type').notNull(),
	rehash: boolean('rehash').default(false).notNull(),
	experimental: boolean('experimental').default(false).notNull(),

	buildNumber: integer('build_number').notNull(),
	jarUrl: varchar('jar_url', { length: 255 }),
	jarSize: integer('jar_size'),
	jarLocation: varchar('jar_location', { length: 51 }),
	zipUrl: varchar('zip_url', { length: 255 }),
	zipSize: integer('zip_size'),

	metadata: jsonb('metadata').notNull(),
	installation: jsonb('installation').$type<InstallStep[][]>().notNull(),
	changes: jsonb('changes').$type<string[]>().notNull(),
	created: timestamp('created')
}, (builds) => [
	index('builds_type_idx').on(builds.type),
	index('builds_experimental_idx').on(builds.experimental),
	index('builds_build_number_idx').on(builds.buildNumber),
	index('builds_jar_url_idx').on(builds.jarUrl).where(isNotNull(builds.jarUrl)),
	index('builds_jar_size_idx').on(builds.jarSize).where(isNotNull(builds.jarSize)),
	index('builds_zip_url_idx').on(builds.zipUrl).where(isNotNull(builds.zipUrl)),
	index('builds_zip_size_idx').on(builds.zipSize).where(isNotNull(builds.zipSize)),
	index('builds_created_idx').on(builds.created),
	index('builds_version_type_idx').on(builds.versionId, builds.type),
	index('builds_project_version_type_idx').on(builds.projectVersionId, builds.type),
	index('builds_version_idx').on(builds.versionId),
	index('builds_changes_idx').on(builds.changes).where(sql`jsonb_array_length(changes) > 0 AND jsonb_array_length(changes) < 10`),

	foreignKey({
		columns: [builds.type, builds.projectVersionId],
		foreignColumns: [projectVersions.type, projectVersions.id],
		name: 'builds_project_version_fk'
	}).onDelete('cascade')
])

export const buildsRelations = relations(builds, ({ one, many }) => ({
	version: one(minecraftVersions, {
		fields: [builds.versionId],
		references: [minecraftVersions.id],
		relationName: 'versions'
	}),

	projectVersion: one(projectVersions, {
		fields: [builds.type, builds.projectVersionId],
		references: [projectVersions.type, projectVersions.id],
		relationName: 'projectVersions'
	}),

	hashes: many(buildHashes)
}))

export const buildHashes = pgTable('build_hashes', {
	buildId: integer('build_id').notNull().references(() => builds.id, { onDelete: 'cascade' }),

	primary: boolean('primary').notNull(),

	sha1: bytea('sha1').notNull(),
	sha224: bytea('sha224').notNull(),
	sha256: bytea('sha256').notNull(),
	sha384: bytea('sha384').notNull(),
	sha512: bytea('sha512').notNull(),
	md5: bytea('md5').notNull()
}, (hashes) => [
	index('buildHashes_build_idx').on(hashes.buildId),
	index('buildHashes_primary_idx').on(hashes.primary),
	index('buildHashes_sha1_idx').on(hashes.sha1).with({ fillfactor: 100 }),
	index('buildHashes_sha224_idx').on(hashes.sha224).with({ fillfactor: 100 }),
	index('buildHashes_sha256_idx').on(hashes.sha256).with({ fillfactor: 100 }),
	index('buildHashes_sha384_idx').on(hashes.sha384).with({ fillfactor: 100 }),
	index('buildHashes_sha512_idx').on(hashes.sha512).with({ fillfactor: 100 }),
	index('buildHashes_md5_idx').on(hashes.md5).with({ fillfactor: 100 })
])

export const buildHashesRelations = relations(buildHashes, ({ one }) => ({
	build: one(builds, {
		fields: [buildHashes.buildId],
		references: [builds.id]
	})
}))

export const configs = pgTable('configs', {
	id: serial('id').primaryKey().notNull(),

	location: varchar('location', { length: 51 }).notNull().unique(),
	type: typesEnum('type').notNull(),
	format: formatsEnum('format').notNull()
}, (configs) => [
	index('configs_type_idx').on(configs.type),
	index('configs_format_idx').on(configs.format)
])

export const configValues = pgTable('config_values', {
	id: serial('id').primaryKey().notNull(),
	configId: integer('config_id').notNull().references(() => configs.id, { onDelete: 'cascade' }),

	sha1: char('sha1', { length: 40 }).notNull(),
	sha224: char('sha224', { length: 56 }).notNull(),
	sha256: char('sha256', { length: 64 }).notNull(),
	sha384: char('sha384', { length: 96 }).notNull(),
	sha512: char('sha512', { length: 128 }).notNull(),
	md5: char('md5', { length: 32 }).notNull(),

	value: text('value').notNull()
}, (configValues) => [
	index('configValues_config_idx').on(configValues.configId),
	uniqueIndex('configValues_unique_config_sha512_idx').on(configValues.configId, configValues.sha512)
])

export const buildConfigs = pgTable('build_configs', {
	buildId: integer('build_id').notNull().references(() => builds.id, { onDelete: 'cascade' }),
	configId: integer('config_id').notNull().references(() => configs.id, { onDelete: 'cascade' }),
	configValueId: integer('config_value_id').notNull().references(() => configValues.id, { onDelete: 'cascade' })
}, (buildConfigs) => [
	primaryKey({ name: 'buildConfigs_pk', columns: [buildConfigs.buildId, buildConfigs.configId] }),

	index('buildConfigs_build_idx').on(buildConfigs.buildId),
	index('buildConfigs_config_idx').on(buildConfigs.configId),
	index('buildConfigs_config_value_idx').on(buildConfigs.configValueId)
])