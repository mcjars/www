import { BASE_URL } from "@/api"
import axios from "axios"

export type ConfigItem = {
	uuid: string
	type: string
	types: string[]
	format: string
	location: string
	aliases: string[]
	builds: number
	values: number
}

export default async function apiGetConfigs(): Promise<ConfigItem[]> {
	const { data } = await axios.get<{ configs: any[] }>(`${BASE_URL}/api/v3/configs`)

	const list = Array.isArray(data?.configs) ? data.configs : []

	return list.map((c: any) => ({
		uuid: c.uuid ?? (c.id !== undefined ? String(c.id) : ''),
		type: c.type,
		types: Array.isArray(c.types)
			? c.types.filter((value: unknown): value is string => typeof value === 'string')
			: typeof c.type === 'string' && c.type
				? [c.type]
				: [],
		format: c.format,
		location: c.location,
		aliases: Array.isArray(c.aliases) ? c.aliases : [],
		builds: Number(c.builds ?? c.build_count ?? 0),
		values: Number(c.values ?? c.value_count ?? 0)
	}))
}
