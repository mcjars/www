import useWebSocket from "react-use-websocket"
import { Badge } from "@/components/ui/badge"
import { Skeleton } from "@/components/ui/skeleton"
import { Card } from "@/components/ui/card"
import useSWR from "swr"
import apiGetTypes from "@/api/types"

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
					{Object.entries(lastJsonMessage.jobs).map(([type, status]) => (
						<Card key={type} className={'w-full h-16 mb-2 p-4 flex flex-row justify-between items-center'}>
							<div className={'flex flex-row items-center'}>
								<img src={`https://s3.mcjars.app/icons/${type.toLowerCase()}.png`} className={'w-10 h-10 rounded-md'} />
								<div className={'flex flex-col ml-4 justify-center'}>
									<p className={'font-bold'}>{type}</p>
									<p className={'text-xs'}>{Object.values(types ?? {}).flat().find((t) => t.identifier === type)?.name}, {Object.values(types ?? {}).flat().find((t) => t.identifier === type)?.builds} builds</p>
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
					))}
				</>
			)}
		</div>
	)
}