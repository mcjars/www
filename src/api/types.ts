import { BASE_URL } from "@/api"
import axios from "axios"

type MinecraftType = {
	identifier: string
	name: string
	icon: string
	color: string
	homepage: string
	description: string
	deprecated: boolean
	experimental: boolean
	categories: string[]
	compatibility: string[]

	builds: number
	versions: {
		minecraft: number
		project: number
	}
}

export default async function apiGetTypes(): Promise<Record<string, MinecraftType[]>> {
	const { data } = await axios.get<{
		types: Record<string, Record<string, MinecraftType>>
	}>(`${BASE_URL}/api/v2/types`)

	return Object.fromEntries(
		Object.entries(data.types)
			.map(([key, types]) => [
				key,
				Object.values(types).map((type) => Object.assign(type, { identifier: type.name.toUpperCase() }))
			])
	)
}