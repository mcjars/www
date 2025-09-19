import logger from "@/globals/logger"
import * as fs from "fs"
import * as path from "path"
import { filesystem } from "@rjweb/utils"
import clickhouse from "@/globals/clickhouse"
import getVersion from "@/index"

logger()
	.text('MCJars API Database', (c) => c.yellowBright)
	.text(`(${process.env.NODE_ENV === 'development' ? 'development' : 'production'} ${getVersion()})`, (c) => c.gray)
	.info()
logger()
	.text('Migrating the clickhouse database now...')
	.info()

async function main() {
	await clickhouse.command({
		query: `
			CREATE TABLE IF NOT EXISTS __migrations (
				\`id\` Int32,
			)
			ENGINE = MergeTree
			ORDER BY (id)
		`
	})

	const { data: migrations } = await clickhouse.query({ query: 'SELECT id FROM __migrations' }).then((r) => r.json<{ id: number }>())

	for await (const migration of filesystem.walk('../clickhouse-migrations', { async: true, recursive: false })) {
		if (migrations.some((m) => m.id === parseInt(migration.name))) continue

		const query = await fs.promises.readFile(path.join('../clickhouse-migrations', migration.name), 'utf8')

		await clickhouse.command({ query })
		await clickhouse.command({
			query: `INSERT INTO __migrations (id) VALUES ({id:Int32})`,
			query_params: { id: parseInt(migration.name) }
		})

		logger()
			.text('Ran migration')
			.text(migration.name, (c) => c.cyan)
			.text('successfully.')
			.info()
	}

	logger()
		.text('All migrations completed', (c) => c.green)
		.info()
}

main().catch(console.error)
