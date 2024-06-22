import axios from "axios"
import { PartialMinecraftBuild } from "@/api/builds"

export default async function apiGetConfig(file: File): Promise<{
	formatted: string
	configs: {
		from: string
		value: string
		accuracy: number
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
	}>('https://versions.mcjars.app/api/v2/config', {
		file: file.name,
		config: await file.text()
	})

	return {
		formatted: data.formatted,
		configs: data.configs
	}
}