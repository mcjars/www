import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiPostUserOrganizationUpdateBuildData(organization: number): Promise<void> {
	await axios.post(`${BASE_URL}/api/user/organizations/${organization}/update-build-data`, undefined, {
		withCredentials: true
	})
}
