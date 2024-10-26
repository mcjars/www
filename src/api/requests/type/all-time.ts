import axios from "axios"

export default async function apiGetTypeRequestsAllTime(type: string) {
	const { data } = await axios.get<{
		requests: {
			root: {
				total: number
				uniqueIps: number
			}

			versions: Record<string, {
				total: number
				uniqueIps: number
			}>
		}
	}>(`https://versions.mcjars.app/api/v2/requests/${type.toUpperCase()}`)

	return data.requests
}