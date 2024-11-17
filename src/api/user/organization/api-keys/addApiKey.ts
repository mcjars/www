import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiAddUserOrganizationApiKey(organization: number, name: string): Promise<string> {
	const { data } = await axios.post<{
		key: string
	}>(`${BASE_URL}/api/user/organizations/${organization}/api-keys`, {
		name
	}, {
		withCredentials: true
	})

	return data.key
}