import { BASE_URL } from "@/api"
import axios from "axios"

export type ApiKey = {
	id: number
	name: string
	created: string
}

export default async function apiGetUserOrganizationApiKeys(organization: number): Promise<ApiKey[]> {
	const { data } = await axios.get<{
		apiKeys: ApiKey[]
	}>(`${BASE_URL}/api/user/organizations/${organization}/api-keys`, {
		withCredentials: true
	})

	return data.apiKeys
}