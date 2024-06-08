import axios from "axios"

type MinecraftVersion = {
	type?: 'RELEASE' | 'SNAPSHOT'
	supported?: boolean
	created?: string
	java?: number
	builds: number
	latest: {
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
}

export default async function apiGetVersions(type: string): Promise<MinecraftVersion[]> {
	const { data } = await axios.get<{
		versions: Record<string, MinecraftVersion>
	}>(`https://mc.rjns.dev/api/v1/builds/${type.toUpperCase()}`)

	return Object.values(data.versions).reverse()
}