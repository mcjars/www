import axios from "axios"

export default async function apiGetTypeLookups() {
	const { data } = await axios.get<{
		types: Record<string, {
			total: number
			uniqueIps: number
		}>
	}>('https://versions.mcjars.app/api/v2/lookups/types')

	return data.types
}