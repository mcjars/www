import axios from "axios"
import { PartialMinecraftBuild } from "@/api/builds"

export default async function apiGetBuild(build: string): Promise<PartialMinecraftBuild> {
	const { data } = await axios.get<{
		build: PartialMinecraftBuild
	}>(`https://versions.mcjars.app/api/v1/build/${build}`)

	return data.build
}