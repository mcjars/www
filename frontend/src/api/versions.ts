import { BASE_URL } from "@/api"
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

export type PaginatedResponse<T> = {
	items: T[]
	page: number
	perPage: number
	hasNextPage: boolean
	total: number | null
}

type ApiGetVersionsOptions = {
	fields?: string[]
	page?: number
	perPage?: number
	search?: string
}

const DEFAULT_VERSION_FIELDS = [
	'type',
	'supported',
	'created',
	'java',
	'builds',
	'latest',
	'project_version_id',
	'version_id'
]

export default async function apiGetVersions(type: string, options?: ApiGetVersionsOptions): Promise<PaginatedResponse<MinecraftVersion>> {
	const page = options?.page ?? 1
	const perPage = options?.perPage ?? 24
	const search = options?.search ?? ''
	const fields = options?.fields ?? DEFAULT_VERSION_FIELDS

	const params = new URLSearchParams()
	for (const field of fields) {
		params.append('fields', field)
	}
	params.set('page', String(page))
	params.set('per_page', String(perPage))
	params.set('search', search)

	const { data } = await axios.get(`${BASE_URL}/api/v3/builds/${type.toUpperCase()}?${params.toString()}`)
	const versionsContainer = data?.versions && typeof data.versions === 'object' && !Array.isArray(data.versions)
		? data.versions
		: undefined

	const versionList = Array.isArray(versionsContainer?.data)
		? versionsContainer.data
		: Array.isArray(data?.versions)
			? data.versions
			: Array.isArray(data?.builds)
				? data.builds
				: data?.builds && typeof data.builds === 'object'
					? Object.values(data.builds)
					: []

	const total =
		typeof versionsContainer?.total === 'number'
			? versionsContainer.total
			: typeof data?.total === 'number'
				? data.total
				: typeof data?.pagination?.total === 'number'
					? data.pagination.total
					: typeof data?.meta?.total === 'number'
						? data.meta.total
						: null

	const responsePage =
		typeof versionsContainer?.page === 'number'
			? versionsContainer.page
			: typeof data?.page === 'number'
				? data.page
				: typeof data?.pagination?.page === 'number'
					? data.pagination.page
					: page

	const responsePerPage =
		typeof versionsContainer?.per_page === 'number'
			? versionsContainer.per_page
			: typeof data?.per_page === 'number'
				? data.per_page
				: typeof data?.pagination?.per_page === 'number'
					? data.pagination.per_page
					: perPage

	const hasNextPage =
		typeof data?.has_next_page === 'boolean'
			? data.has_next_page
			: typeof data?.pagination?.has_next_page === 'boolean'
				? data.pagination.has_next_page
				: total !== null
					? responsePage * responsePerPage < total
					: versionList.length >= responsePerPage

	const normalizedVersionList = (Array.isArray(versionList) ? versionList : []).map((v: any) => ({
		type: v.type,
		supported: Boolean(v.supported),
		created: v.created,
		java: v.java,
		builds: typeof v.builds === 'number' ? v.builds : (typeof v.build_count === 'number' ? v.build_count : 0),
		latest: {
			projectVersionId: v.latest?.projectVersionId ?? v.latest?.project_version_id ?? v.projectVersionId ?? v.project_version_id ?? null,
			versionId: v.latest?.versionId ?? v.latest?.version_id ?? v.versionId ?? v.version_id ?? null
		}
	}))

	return {
		items: normalizedVersionList as MinecraftVersion[],
		page: responsePage,
		perPage: responsePerPage,
		hasNextPage,
		total
	}
}