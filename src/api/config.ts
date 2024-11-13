import { BASE_URL } from "@/api"
import axios from "axios"
import { PartialMinecraftBuild } from "@/api/builds"

export default async function apiGetConfig(file: File): Promise<{
	formatted: string
	configs: {
		from: string
		value: string
		build: PartialMinecraftBuild | null
	}[]
}> {
	const { data } = await axios.post<{
		formatted: string
		configs: {
			from: string
			value: string
			accuracy: number
			build: PartialMinecraftBuild
		}[]
	}>(`${BASE_URL}/api/v2/config`, {
		file: file.name.replace(/\(\d+\)/g, ''),
		config: await file.text()
	})

	return {
		formatted: data.formatted,
		configs: data.configs
	}
}