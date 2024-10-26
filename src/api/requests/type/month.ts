import axios from "axios"

export default async function apiGetTypeRequestsMonth(type: string, year: number, month: number) {
	const { data } = await axios.get<{
		requests: {
			day: number
			root: {
				total: number
				uniqueIps: number
			}

			versions: Record<string, {
				total: number
				uniqueIps: number
			}>
		}[]
	}>(`https://versions.mcjars.app/api/v2/requests/${type.toUpperCase()}/history/${year}/${month}`)

	return data.requests
}