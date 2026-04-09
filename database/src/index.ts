import logger from "@/globals/logger"
import * as fs from "fs"
import { system } from "@rjweb/utils"
import database from "./globals/database"
import { eq } from "drizzle-orm"

export default function getVersion() {
	return `${JSON.parse(fs.readFileSync('../package.json', 'utf8')).version}:${system.execute('git rev-parse --short=10 HEAD').trim()}`
}

if (process.argv[2] === 'migrate-clickhouse') {
	require('@/migrate-clickhouse')
} else {
	logger()
		.text('MCJars API Database', (c) => c.yellowBright)
		.text(`(${process.env.NODE_ENV === 'development' ? 'development' : 'production'} ${getVersion()})`, (c) => c.gray)
		.info()
	logger()
		.text('This is not meant to be ran directly, this only provides the database schema (and connection for build backend)', (c) => c.red)
		.info()
}

async function main() {
	const configValues = await database.select().from(database.schema.configValues).innerJoin(database.schema.configs, eq(database.schema.configValues.configId, database.schema.configs.id));

	const promises = []

	for (const configValue of configValues.filter((c) => Object.keys(c.config_values.parsed).length <= 0)) {
		const parsed = database.parseConfig(configValue.configs.location, configValue.config_values.value);

		console.log(configValue.configs.location)

		promises.push(database.$client.query(`UPDATE config_values SET parsed = $1 WHERE id = $2`, [parsed || {}, configValue.config_values.id]))
	}

	await Promise.all(promises)
}

if (require.main === module) {
	main().catch((err) => {
		console.error(err)
		process.exit(1)
	})
}