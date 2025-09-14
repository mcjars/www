import { drizzle } from "drizzle-orm/node-postgres"
import * as schema from "@@/schema"
import env from "@@/globals/env"
import yaml from "js-yaml"
import json5 from "json5"
import logger from "@@/globals/logger"
import { Pool } from "pg"

export const configs: Record<string, {
	type: schema.ServerType
	format: schema.Format
	aliases: string[]
}> = {
	// Vanilla
	'server.properties': {
		type: 'VANILLA',
		format: 'PROPERTIES',
		aliases: ['server.properties']
	},

	// Spigot
	'spigot.yml': {
		type: 'SPIGOT',
		format: 'YAML',
		aliases: ['spigot.yml']
	}, 'bukkit.yml': {
		type: 'SPIGOT',
		format: 'YAML',
		aliases: ['bukkit.yml']
	},

	// Paper
	'paper.yml': {
		type: 'PAPER',
		format: 'YAML',
		aliases: ['paper.yml']
	}, 'config/paper-global.yml': {
		type: 'PAPER',
		format: 'YAML',
		aliases: ['config/paper-global.yml', 'paper-global.yml']
	}, 'config/paper-world-defaults.yml': {
		type: 'PAPER',
		format: 'YAML',
		aliases: ['config/paper-world-defaults.yml', 'paper-world-defaults.yml']
	},

	// Pufferfish
	'pufferfish.yml': {
		type: 'PUFFERFISH',
		format: 'YAML',
		aliases: ['pufferfish.yml']
	},

	// Purpur
	'purpur.yml': {
		type: 'PURPUR',
		format: 'YAML',
		aliases: ['purpur.yml']
	},

	// Leaves
	'leaves.yml': {
		type: 'LEAVES',
		format: 'YAML',
		aliases: ['leaves.yml']
	},

	// Canvas
	'canvas.yml': {
		type: 'CANVAS',
		format: 'YAML',
		aliases: ['canvas.yml']
	}, 'config/canvas-server.json5': {
		type: 'CANVAS',
		format: 'JSON5',
		aliases: ['config/canvas-server.json5', 'canvas-server.json5']
	},

	// DivineMC
	'divinemc.yml': {
		type: 'DIVINEMC',
		format: 'YAML',
		aliases: ['divinemc.yml']
	},

	// Sponge
	'config/sponge/global.conf': {
		type: 'SPONGE',
		format: 'CONF',
		aliases: ['config/sponge/global.conf', 'global.conf']
	}, 'config/sponge/sponge.conf': {
		type: 'SPONGE',
		format: 'CONF',
		aliases: ['config/sponge/sponge.conf', 'sponge.conf']
	}, 'config/sponge/tracker.conf': {
		type: 'SPONGE',
		format: 'CONF',
		aliases: ['config/sponge/tracker.conf', 'tracker.conf']
	},

	// Arclight
	'arclight.conf': {
		type: 'ARCLIGHT',
		format: 'CONF',
		aliases: ['arclight.conf']
	},

	// NeoForge
	'config/neoforge-server.toml': {
		type: 'NEOFORGE',
		format: 'TOML',
		aliases: ['config/neoforge-server.toml', 'neoforge-server.toml']
	}, 'config/neoforge-common.toml': {
		type: 'NEOFORGE',
		format: 'TOML',
		aliases: ['config/neoforge-common.toml', 'neoforge-common.toml']
	},

	// Mohist
	'mohist-config/mohist.yml': {
		type: 'MOHIST',
		format: 'YAML',
		aliases: ['mohist-config/mohist.yml', 'mohist.yml']
	},

	// Velocity
	'velocity.toml': {
		type: 'VELOCITY',
		format: 'TOML',
		aliases: ['velocity.toml']
	},

	// BungeeCord
	'config.yml': {
		type: 'BUNGEECORD',
		format: 'YAML',
		aliases: ['config.yml']
	},

	// Waterfall
	'waterfall.yml': {
		type: 'WATERFALL',
		format: 'YAML',
		aliases: ['waterfall.yml']
	},

	// NanoLimbo
	'settings.yml': {
		type: 'NANOLIMBO',
		format: 'YAML',
		aliases: ['settings.yml']
	},

	// Loohp Limbo // TODO: Find better way to handle this
	// 'server.properties': {
	// 	type: 'LOOHP_LIMBO',
	// 	format: 'PROPERTIES',
	// 	aliases: ['server.properties']
	// }

	// Magma
	'magma.yml': {
		type: 'MAGMA',
		format: 'YAML',
		aliases: ['magma.yml']
	},

	// Leaf
	'config/leaf-global.yml': {
		type: 'LEAF',
		format: 'YAML',
		aliases: ['config/leaf-global.yml', 'leaf-global.yml']
	}, 'config/gale-global.yml': {
		type: 'LEAF',
		format: 'YAML',
		aliases: ['config/gale-global.yml', 'gale-global.yml']
	}, 'config/gale-world-defaults.yml': {
		type: 'LEAF',
		format: 'YAML',
		aliases: ['config/gale-world-defaults.yml', 'gale-world-defaults.yml']
	}
}

const pool = new Pool({
	connectionString: env.DATABASE_URL
})

const db = drizzle(pool, { schema }),
	startTime = performance.now()

db.$client.connect().then(() => {
	logger()
		.text('Database', (c) => c.cyan)
		.text('Connection established!')
		.text(`(${(performance.now() - startTime).toFixed(1)}ms)`, (c) => c.gray)
		.info()
})

export default Object.assign(db, {
	schema,

	formatConfig(file: string, rawValue: string) {
		let value = ''

		for (const line of rawValue.trim().split('\n')) {
			if (line.trimStart()[0] === '#' || !line.trim().length) continue

			value += line + '\n'
		}

		if (file.endsWith('.properties')) {
			value = value.trim().split('\n')
				.sort()
				.join('\n')
		} else if (file.endsWith('.yml') || file.endsWith('.yaml')) {			
			const quotedDecimalRegex = /: '([-+]?[0-9]*\.[0-9]+)'/g
			const quotedReplacements = new Map()
			let quotedCounter = 0
			
			value = value.replace(quotedDecimalRegex, (_, decimalValue) => {
				const placeholder = `__QUOTED_DECIMAL_${quotedCounter++}__`
				quotedReplacements.set(placeholder, decimalValue)
				return `: ${placeholder}`
			})
			
			const unquotedDecimalRegex = /: ([-+]?[0-9]*\.[0-9]+)(\s|$)/g
			const unquotedReplacements = new Map()
			let unquotedCounter = 0
			
			value = value.replace(unquotedDecimalRegex, (_, decimalValue, ending) => {
				const placeholder = `__UNQUOTED_DECIMAL_${unquotedCounter++}__`
				unquotedReplacements.set(placeholder, decimalValue)
				return `: ${placeholder}${ending}`
			})
			
			const loadedData = yaml.load(value)
			
			value = yaml.dump(loadedData, { 
				sortKeys: true,
				noArrayIndent: true,
				lineWidth: Infinity
			})
			
			quotedReplacements.forEach((decimalValue, placeholder) => {
				value = value.replace(new RegExp(`${placeholder}`, 'g'), `'${decimalValue}'`)
			})
			
			unquotedReplacements.forEach((decimalValue, placeholder) => {
				value = value.replace(new RegExp(`${placeholder}`, 'g'), decimalValue)
			})
		} else if (file.endsWith('.json') || file.endsWith('.json5')) {
			const loadedData: object = json5.parse(value)

			function sortAndMask(obj: any): any {
				if (Array.isArray(obj)) {
					return obj.map(sortAndMask)
				} else if (obj && typeof obj === 'object') {
					const sorted: any = {}
					for (const key of Object.keys(obj).sort()) {
						let val = obj[key]
						if (typeof val === 'object' && val !== null) {
							val = sortAndMask(val)
						} else if (key.startsWith('seed')) {
							val = 'xxx'
						}
						sorted[key] = val
					}

					return sorted
				}

				return obj
			}

			const sortedMasked = sortAndMask(loadedData)
			value = JSON.stringify(sortedMasked, null, 2)
		}

		if (file === 'velocity.toml') {
			value = value.replace(/forwarding-secret = "(.*)"/, 'forwarding-secret = "xxx"')
		}

		if (file === 'config.yml') {
			value = value
				.replace(/stats_uuid: (.*)/, 'stats_uuid: xxx')
				.replace(/stats: (.*)/, 'stats: xxx')
		}

		if (file === 'leaves.yml') {
			value = value.replace(/server-id: (.*)/, 'server-id: xxx')
		}

		value = value
			.replace(/seed-(.*)=(.*)/g, 'seed-$1=xxx')
			.replace(/seed-(.*): (.*)/g, 'seed-$1: xxx')

		return value
	}
})
