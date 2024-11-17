import { BASE_URL } from "@/api"
import axios from "axios"

export type User = {
	id: number
	name: string | null
	email: string
	login: string
	avatar: string
}

export default async function apiGetUserInfos(): Promise<User | null> {
	const { data } = await axios.get<{
		user: User
	}>(`${BASE_URL}/api/user`, {
		withCredentials: true
	}).catch(() => ({ data: { user: null } }))

	return data.user
}