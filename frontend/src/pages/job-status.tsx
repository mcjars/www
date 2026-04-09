import useWebSocket from "react-use-websocket"
import { Badge } from "@/components/ui/badge"
import { Skeleton } from "@/components/ui/skeleton"
import { Card } from "@/components/ui/card"
import useSWR from "swr"
import apiGetTypes from "@/api/types"
import { useMemo } from "react"

type WebSocketEvent = {
	jobs: Record<string, {
		status: 'running' | 'idle' | 'waiting'

		checked: number
		updated: number
		created: number
	}>
}

export default function PageJobStatus() {
	const { lastJsonMessage } = useWebSocket<WebSocketEvent>(`wss://backend.mcjars.app/api/jobs/ws`, {
		retryOnError: true,
		shouldReconnect: () => true
	})

	const { data: types } = useSWR(
		['types'],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const knownTypes = useMemo(() => Object.values(types ?? {}).flat(), [types])
	const normalizeType = (value: string) => value.trim().toUpperCase()

	const entries = useMemo(() => {
		if (!lastJsonMessage?.jobs) return [] as Array<[string, WebSocketEvent['jobs'][string]]>

		const jobsByType = Object.entries(lastJsonMessage.jobs)
		const indexByType = new Map(jobsByType.map((entry) => [normalizeType(entry[0]), entry]))

		const knownOrdered = knownTypes
			.map((knownType) => indexByType.get(normalizeType(knownType.identifier)))
			.filter((entry): entry is [string, WebSocketEvent['jobs'][string]] => Boolean(entry))

		const knownKeys = new Set(knownOrdered.map(([type]) => normalizeType(type)))
		const unknown = jobsByType.filter(([type]) => !knownKeys.has(normalizeType(type)))

		return [...knownOrdered, ...unknown]
	}, [lastJsonMessage, knownTypes])

	const getTypeData = (type: string) => knownTypes.find((entry) => normalizeType(entry.identifier) === normalizeType(type))

	return (
		<div className={'w-full flex flex-col items-center'}>
			{!lastJsonMessage ? (
				<>
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
					<Skeleton className={'w-full h-16 rounded-xl mb-2'} />
				</>
			) : (
				<>
					{entries.map(([type, status]) => {
						const typeData = getTypeData(type)
						const displayName = typeData?.name ?? type.replace(/_/g, ' ')
						const icon = typeData?.icon ?? `https://s3.mcjars.app/icons/${type.toLowerCase()}.png`

						return (
							<Card key={type} className={'w-full h-16 mb-2 p-4 flex flex-row justify-between items-center'}>
								<div className={'flex flex-row items-center'}>
									<img src={icon} alt={displayName} className={'w-10 h-10 rounded-md'} />
									<div className={'flex flex-col ml-4 justify-center'}>
										<p className={'font-bold'}>{type}</p>
										<p className={'text-xs'}>{displayName}, {typeData?.builds ?? 0} builds</p>
									</div>
								</div>

								<div className={'flex flex-row items-center'}>
									<Card className={'p-2 hidden md:flex flex-row items-center justify-between w-32 mr-4'}>
										<p className={'font-bold'}>{status.created}</p>
										<p className={'text-xs'}>Created</p>
									</Card>
									<Card className={'p-2 hidden md:flex flex-row items-center justify-between w-32 mr-4'}>
										<p className={'font-bold'}>{status.updated}</p>
										<p className={'text-xs'}>Updated</p>
									</Card>
									<Card className={'p-2 hidden md:flex flex-row items-center justify-between w-32 mr-4'}>
										<p className={'font-bold'}>{status.checked}</p>
										<p className={'text-xs'}>Checked</p>
									</Card>

									{status.status === 'running' ? (
										<Badge className={'bg-green-400 hover:bg-green-300'}>Running</Badge>
									) : status.status === 'idle' ? (
										<Badge className={'bg-blue-400 hover:bg-blue-300'}>Idle</Badge>
									) : (
										<Badge className={'bg-yellow-400 hover:bg-yellow-300'}>Waiting</Badge>
									)}
								</div>
							</Card>
						)
					})}
				</>
			)}
		</div>
	)
}