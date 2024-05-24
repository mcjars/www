import axios from "axios"

export type PartialMinecraftBuild = {
	id: number
	type: string
	projectVersionId: string | null
	versionId: string | null
	buildNumber: number
	created: string
	jarUrl: string | null
	jarSize: number | null
	zipUrl: string | null
	zipSize: number | null
}

export default async function apiGetBuilds(type: string, version: string): Promise<PartialMinecraftBuild[]> {
	const { data } = await axios.get<{
		builds: PartialMinecraftBuild[]
	}>(`https://mc.rjns.dev/api/v1/builds/${type.toUpperCase()}/${version}`)

	return data.builds
}