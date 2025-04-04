import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiDeleteUserOrganizationSubuser(organization: number, login: string): Promise<void> {
	await axios.delete(`${BASE_URL}/api/user/organizations/${organization}/subusers/${login}`, {
		withCredentials: true
	})
}