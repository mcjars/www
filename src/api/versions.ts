import axios from "axios"

type MinecraftVersion = {
	type: 'RELEASE' | 'SNAPSHOT'
	supported: boolean
	created: string
	java: number
	builds: number
	latest: {
		projectVersionId: string | null
		versionId: string | null
	}
}

export default async function apiGetVersions(type: string): Promise<MinecraftVersion[]> {
	const { data } = await axios.get<{
		builds: Record<string, MinecraftVersion>
	}>(`https://versions.mcjars.app/api/v2/builds/${type.toUpperCase()}?fields=projectVersionId,versionId`)

	return Object.values(data.builds).reverse()
}