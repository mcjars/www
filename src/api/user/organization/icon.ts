import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiPostUserOrganizationIcon(organization: number, icon: File): Promise<string> {
	const { data } = await axios.post<{ url: string }>(`${BASE_URL}/api/user/organizations/${organization}/icon`, icon, {
		withCredentials: true
	})

	return data.url
}