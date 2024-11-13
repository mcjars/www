import { BASE_URL } from "@/api"
import axios from "axios"
import { PartialMinecraftBuild } from "@/api/builds"

export default async function apiGetBuild(build: string): Promise<PartialMinecraftBuild> {
	const { data } = await axios.post<{
		build: PartialMinecraftBuild
	}>(`${BASE_URL}/api/v2/build`, {
		hash: {
			sha256: build
		}
	})

	return data.build
}