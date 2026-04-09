import { BASE_URL } from "@/api"
import axios from "axios"

export default async function apiPostConfigFormat(file: File, rawConfig?: string): Promise<string> {
	const { data } = await axios.post<{ formatted?: string }>(`${BASE_URL}/api/v3/configs/format`, {
		file: file.name.replace(/\(\d+\)/g, ""),
		config: rawConfig ?? await file.text()
	})

	return typeof data?.formatted === "string" ? data.formatted : ""
}