import axios from "axios"

export default async function apiGetVersionRequestsMonth(version: string, year: number, month: number) {
	const { data } = await axios.get<{
		requests: Record<string, {
			day: number
			total: number
			uniqueIps: number
		}[]>
	}>(`https://versions.mcjars.app/api/v2/requests/version/${version}/history/${year}/${month}`)

	return {
		type: 'month',
		requests: data.requests
	} as const
}