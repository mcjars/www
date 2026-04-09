import { BASE_URL } from "@/api"
import { PartialMinecraftBuild } from "@/api/builds"
import axios from "axios"

export type TypeVersionSummary = {
	versionId: string
	projectVersionId: string | null
	created: string | null
	builds: number
}

const normalizeBuild = (build: any): PartialMinecraftBuild => ({
	uuid: build.uuid ?? (build.id !== undefined ? String(build.id) : ""),
	type: build.type ?? "UNKNOWN",
	projectVersionId: build.project_version_id ?? build.projectVersionId ?? null,
	versionId: build.version_id ?? build.versionId ?? null,
	name: build.name ?? "",
	experimental: Boolean(build.experimental),
	created: build.created ?? null,
	changes: Array.isArray(build.changes) ? build.changes : [],
	installation: Array.isArray(build.installation) ? build.installation : []
})

const getArray = (value: any): any[] => {
	if (Array.isArray(value)) return value
	if (value && typeof value === "object" && Array.isArray(value.data)) return value.data
	return []
}

export default async function apiGetTypeVersions(type: string): Promise<TypeVersionSummary[]> {
	const { data } = await axios.get(`${BASE_URL}/api/v3/builds/types/${type.toUpperCase()}/versions`)
	const versions = getArray(data?.versions ?? data?.items ?? data?.data)

	return versions
		.map((version: any) => ({
			versionId:
				version.latest?.version_id,
			projectVersionId:
				version.latest?.project_version_id,
			created: version.latest?.created ?? null,
			builds: Number(version.builds)
		}))
		.filter((version: TypeVersionSummary) => Boolean(version.versionId))
}

export async function apiGetTypeVersionBuilds(type: string, version: string): Promise<PartialMinecraftBuild[]> {
	const { data } = await axios.get(`${BASE_URL}/api/v3/builds/types/${type.toUpperCase()}/versions/${encodeURIComponent(version)}`)
	const builds = getArray(data?.builds ?? data?.items ?? data?.data)

	return builds.map((build: any) => normalizeBuild(build))
}
