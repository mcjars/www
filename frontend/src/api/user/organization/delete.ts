import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiDeleteUserOrganization(organization: number): Promise<void> {
	await axios.delete(`${BASE_URL}/api/user/organizations/${organization}`, {
		withCredentials: true
	})
}