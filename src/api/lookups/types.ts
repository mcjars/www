import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiGetTypeLookups() {
	const { data } = await axios.get<{
		types: Record<string, {
			total: number
			uniqueIps: number
		}>
	}>(`${BASE_URL}/api/v2/lookups/types`)

	return data.types
}