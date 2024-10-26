import axios from "axios"

export default async function apiGetTypeVersionLookupsAllTime(type: string) {
	const { data } = await axios.get<{
		versions: Record<string, {
			total: number
			uniqueIps: number
		}>
	}>(`https://versions.mcjars.app/api/v2/lookups/versions/${type.toUpperCase()}`)

	return data.versions
}