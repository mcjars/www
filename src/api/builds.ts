import axios from "axios"

export type PartialMinecraftBuild = {
	id: number
	type: string
	projectVersionId: string | null
	versionId: string | null
	buildNumber: number
	experimental: boolean
	created: string
	jarUrl: string | null
	jarSize: number | null
	jarLocation: string | null
	zipUrl: string | null
	zipSize: number | null
	changes: string[]
}

export default async function apiGetBuilds(type: string, version: string): Promise<PartialMinecraftBuild[]> {
	const { data } = await axios.get<{
		builds: PartialMinecraftBuild[]
	}>(`https://mc.rjns.dev/api/v2/builds/${type.toUpperCase()}/${version}`)

	return data.builds
}