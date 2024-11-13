import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiGetTypeStats(type: string) {
	const { data } = await axios.get<{
		stats: {
			size: {
				total: {
					jar: number
					zip: number
				}
			}
		}
	}>(`${BASE_URL}/api/v2/stats/${type.toUpperCase()}`)

	return data.stats
}