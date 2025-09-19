import env from "@/globals/env"
import logger from "@/globals/logger"
import { createClient } from "@clickhouse/client"

const clickhouse = createClient({
		url: env.CLICKHOUSE_URL,
		database: env.CLICKHOUSE_DATABASE,
		username: env.CLICKHOUSE_USERNAME,
		password: env.CLICKHOUSE_PASSWORD
	}),
	startTime = performance.now()

clickhouse.ping().then(() => {
	logger()
		.text('Clickhouse', (c) => c.redBright)
		.text('Connection established!')
		.text(`(${(performance.now() - startTime).toFixed(1)}ms)`, (c) => c.gray)
		.info()
})

export default clickhouse
