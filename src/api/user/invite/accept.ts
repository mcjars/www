import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiPostUserIniteAccept(organizationId: number): Promise<void> {
	await axios.post(`${BASE_URL}/api/user/invites/${organizationId}/accept`, {}, {
		withCredentials: true
	})
}