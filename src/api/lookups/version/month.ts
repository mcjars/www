import axios from "axios"

export default async function apiGetTypeVersionLookupsMonth(type: string, year: number, month: number) {
	const { data } = await axios.get<{
		versions: Record<string, {
			day: number
			total: number
			uniqueIps: number
		}[]>
	}>(`https://versions.mcjars.app/api/v2/lookups/versions/${type.toUpperCase()}/history/${year}/${month}`)

	return data.versions
}