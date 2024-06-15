import axios from "axios"

type APIStats = {
	builds: number
	hashes: number
	requests: number
}

export default async function apiGetStats(): Promise<APIStats> {
	const { data } = await axios.get<{
		stats: APIStats
	}>('https://versions.mcjars.app/api/v1/stats')

	return data.stats
}