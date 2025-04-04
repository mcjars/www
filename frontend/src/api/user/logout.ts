import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiPostUserLogout(): Promise<void> {
	await axios.post(`${BASE_URL}/api/user/logout`, {}, {
		withCredentials: true
	})
}