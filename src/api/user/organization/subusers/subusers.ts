import { BASE_URL } from "@/api"
import axios from "axios"
import { User } from "@/api/user/infos"

export default async function apiGetUserOrganizationSubusers(organization: number): Promise<User[]> {
	const { data } = await axios.get<{
		users: User[]
	}>(`${BASE_URL}/api/user/organizations/${organization}/subusers`, {
		withCredentials: true
	})

	return data.users
}