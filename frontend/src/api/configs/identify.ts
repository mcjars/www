import { BASE_URL } from "@/api"
import axios from "axios"
import { PartialMinecraftBuild } from "@/api/builds"

const normalizeBuild = (build: any): PartialMinecraftBuild | null => {
	if (!build || typeof build !== 'object') return null

	return {
		uuid: build.uuid ?? (build.id !== undefined ? String(build.id) : ''),
		type: build.type ?? 'UNKNOWN',
		projectVersionId: build.project_version_id ?? build.projectVersionId ?? null,
		versionId: build.version_id ?? build.versionId ?? null,
		name: build.name ?? '',
		experimental: Boolean(build.experimental),
		created: build.created ?? null,
		changes: Array.isArray(build.changes) ? build.changes : [],
		installation: Array.isArray(build.installation) ? build.installation : []
	}
}

export default async function apiGetConfigSearch(file: File): Promise<{
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
			accuracy?: number
			build: any | null
		}[]
	}>(`${BASE_URL}/api/v3/configs/identify`, {
		file: file.name.replace(/\(\d+\)/g, ''),
		config: await file.text()
	})

	return {
		formatted: typeof data?.formatted === 'string' ? data.formatted : '',
		configs: Array.isArray(data?.configs)
			? data.configs.map((item) => ({
				from: item?.from ?? 'UNKNOWN',
				value: item?.value ?? '',
				build: normalizeBuild(item?.build)
			}))
			: []
	}
}