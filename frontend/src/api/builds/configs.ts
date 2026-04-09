import { BASE_URL } from "@/api"
import axios from "axios"

export type BuildConfigItem = {
	configUuid: string
	valueUuid: string
	location: string
	type: string
	format: string
	value: string
}

export default async function apiGetBuildConfigs(build: string): Promise<BuildConfigItem[]> {
	const { data } = await axios.get<{ configs: any[] }>(`${BASE_URL}/api/v3/builds/${build}/configs`)

	return Array.isArray(data?.configs)
		? data.configs.map((config: any) => ({
			configUuid: config.config_uuid ?? config.configUuid ?? (config.id !== undefined ? String(config.id) : ""),
			valueUuid: config.value_uuid ?? config.valueUuid ?? "",
			location: config.location ?? "",
			type: config.type ?? "UNKNOWN",
			format: config.format ?? "UNKNOWN",
			value: config.value ?? ""
		}))
		: []
}