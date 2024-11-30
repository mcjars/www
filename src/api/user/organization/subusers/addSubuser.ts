import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiAddUserOrganizationSubuser(organization: number, login: string): Promise<void> {
	await axios.post(`${BASE_URL}/api/user/organizations/${organization}/subusers`, {
		login: login.replace('@', '')
	}, {
		withCredentials: true
	})
}