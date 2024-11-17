import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiGetUserOrganizationStats(organization: number): Promise<Record<'requests' | 'userAgents' | 'ips' | 'origins' | 'continents' | 'countries', number>> {
	const { data } = await axios.get<{
		stats: Record<'requests' | 'userAgents' | 'ips' | 'origins' | 'continents' | 'countries', number>
	}>(`${BASE_URL}/api/user/organizations/${organization}/stats`, {
		withCredentials: true
	})

	return data.stats
}