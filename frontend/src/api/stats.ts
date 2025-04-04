import { BASE_URL } from "@/api"
import axios from "axios"

type APIStats = {
	builds: number
	hashes: number
	requests: number

	size: {
		database: number
	}

	total: {
		jarSize: number
		zipSize: number
	}
}

export default async function apiGetStats(): Promise<APIStats> {
	const { data } = await axios.get<{
		stats: APIStats
	}>(`${BASE_URL}/api/v1/stats`)

	return data.stats
}