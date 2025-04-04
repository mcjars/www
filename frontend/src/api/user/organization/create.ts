import { BASE_URL } from "@/api"
import axios from "axios"

export type CreateOrganizationData = {
	name: string
}

export default async function apiCreateUserOrganization(data: CreateOrganizationData): Promise<void> {
	await axios.post(`${BASE_URL}/api/user/organizations`, data, {
		withCredentials: true
	})
}