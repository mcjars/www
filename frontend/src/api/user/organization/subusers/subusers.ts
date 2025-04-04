import { BASE_URL } from "@/api"
import axios from "axios"
import { User } from "@/api/user/infos"

export default async function apiGetUserOrganizationSubusers(organization: number): Promise<{
	user: User
	pending: boolean
	created: string
}[]> {
	const { data } = await axios.get<{
		users: {
			user: User
			pending: boolean
			created: string
		}[]
	}>(`${BASE_URL}/api/user/organizations/${organization}/subusers`, {
		withCredentials: true
	})

	return data.users
}