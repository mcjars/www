import axios from "axios"

export type InstallStep = {
	type: 'download'

	file: string
	url: string
	size: number
} | {
	type: 'unzip'

	file: string
	location: string
} | {
	type: 'remove'

	location: string
}

export type PartialMinecraftBuild = {
	id: number
	type: string
	projectVersionId: string | null
	versionId: string | null
	buildNumber: number
	experimental: boolean
	created: string | null
	jarUrl: string | null
	jarSize: number | null
	jarLocation: string | null
	zipUrl: string | null
	zipSize: number | null
	changes: string[]
	installation: InstallStep[][]
}

export default async function apiGetBuilds(type: string, version: string): Promise<PartialMinecraftBuild[]> {
	const { data } = await axios.get<{
		builds: PartialMinecraftBuild[]
	}>(`https://versions.mcjars.app/api/v2/builds/${type.toUpperCase()}/${version}`)

	return data.builds
}