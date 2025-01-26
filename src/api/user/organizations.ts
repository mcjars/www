import { BASE_URL } from "@/api"
import axios from "axios"
import { User } from "@/api/user/infos"

export type Organization = {
	id: number
	name: string
	icon: string | null
	types: string[]
	created: string
	owner: User
}

export default async function apiGetUserOrganizations(): Promise<Record<'owned' | 'member' | 'invites', Organization[]>> {
	const { data } = await axios.get<{
		organizations: Record<'owned' | 'member' | 'invites', Organization[]>
	}>(`${BASE_URL}/api/user/organizations`, {
		withCredentials: true
	})

	return data.organizations
}