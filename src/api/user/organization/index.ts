import { BASE_URL } from "@/api"
import axios from "axios"

export type PatchOrganizationData = {
	name?: string
	types?: string[]
	owner?: string
	public?: boolean
}

export default async function apiPatchUserOrganization(organization: number, data: PatchOrganizationData): Promise<void> {
	await axios.patch(`${BASE_URL}/api/user/organizations/${organization}`, data, {
		withCredentials: true
	})
}