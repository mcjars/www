import { PaginatedResponse } from "@/api/versions"
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
	uuid: string
	type: string
	projectVersionId: string | null
	versionId: string | null
	name: string
	experimental: boolean
	created: string | null
	changes: string[]
	installation: InstallStep[][]
}

type ApiGetBuildsOptions = {
	fields?: string[]
	page?: number
	perPage?: number
	search?: string
}

const DEFAULT_BUILD_FIELDS = [
	'uuid',
	'type',
	'project_version_id',
	'version_id',
	'name',
	'experimental',
	'created',
	'changes',
	'installation'
]

export default async function apiGetBuilds(type: string, version: string, options?: ApiGetBuildsOptions): Promise<PaginatedResponse<PartialMinecraftBuild>> {
	const page = options?.page ?? 1
	const perPage = options?.perPage ?? 25
	const search = options?.search ?? ''
	const fields = options?.fields ?? DEFAULT_BUILD_FIELDS

	const params = new URLSearchParams()
	for (const field of fields) {
		params.append('fields[]', field)
	}
	params.set('page', String(page))
	params.set('per_page', String(perPage))
	params.set('search', search)

	const { data } = await axios.get(`${BASE_URL}/api/v3/builds/${type.toUpperCase()}/${version}?${params.toString()}`)
	const buildsContainer = data?.builds && typeof data.builds === 'object' && !Array.isArray(data.builds)
		? data.builds
		: undefined

	const buildList = Array.isArray(buildsContainer?.data)
		? buildsContainer.data
		: Array.isArray(data?.builds)
			? data.builds
			: Array.isArray(data?.items)
				? data.items
				: []

	const normalizedBuildList = (Array.isArray(buildList) ? buildList : []).map((b: any) => ({
		uuid: b.uuid ?? (b.id !== undefined ? String(b.id) : ''),
		type: b.type,
		projectVersionId: b.project_version_id ?? b.projectVersionId ?? null,
		versionId: b.version_id ?? b.versionId ?? null,
		name: b.name,
		experimental: Boolean(b.experimental),
		created: b.created ?? null,
		changes: Array.isArray(b.changes) ? b.changes : [],
		installation: Array.isArray(b.installation) ? b.installation : []
	}))

	const total =
		typeof buildsContainer?.total === 'number'
			? buildsContainer.total
			: typeof data?.total === 'number'
				? data.total
				: typeof data?.pagination?.total === 'number'
					? data.pagination.total
					: typeof data?.meta?.total === 'number'
						? data.meta.total
						: null

	const responsePage =
		typeof buildsContainer?.page === 'number'
			? buildsContainer.page
			: typeof data?.page === 'number'
				? data.page
				: typeof data?.pagination?.page === 'number'
					? data.pagination.page
					: page

	const responsePerPage =
		typeof buildsContainer?.per_page === 'number'
			? buildsContainer.per_page
			: typeof data?.per_page === 'number'
				? data.per_page
				: typeof data?.pagination?.per_page === 'number'
					? data.pagination.per_page
					: perPage

	const hasNextPage =
		typeof buildsContainer?.has_next_page === 'boolean'
			? buildsContainer.has_next_page
			: typeof data?.has_next_page === 'boolean'
				? data.has_next_page
				: typeof data?.pagination?.has_next_page === 'boolean'
					? data.pagination.has_next_page
					: total !== null
						? responsePage * responsePerPage < total
						: buildList.length >= responsePerPage

	return {
		items: normalizedBuildList as PartialMinecraftBuild[],
		page: responsePage,
		perPage: responsePerPage,
		hasNextPage,
		total
	}
}