import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiGetVersionLookups() {
	const { data } = await axios.get<{
		versions: Record<string, {
			total: number
			uniqueIps: number
		}>
	}>(`${BASE_URL}/api/v2/lookups/versions`)

	return data.versions
}