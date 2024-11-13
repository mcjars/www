import { BASE_URL } from "@/api"
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
	name: string
	experimental: boolean
	created: string | null
	changes: string[]
	installation: InstallStep[][]
}

export default async function apiGetBuilds(type: string, version: string): Promise<PartialMinecraftBuild[]> {
	const { data } = await axios.get<{
		builds: PartialMinecraftBuild[]
	}>(`${BASE_URL}/api/v2/builds/${type.toUpperCase()}/${version}?fields=id,type,projectVersionId,versionId,name,experimental,created,changes,installation`)

	return data.builds
}