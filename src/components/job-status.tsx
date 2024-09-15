import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import useWebSocket from "react-use-websocket"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"

type WebSocketEvent = {
	jobs: Record<string, {
		running: boolean

		checked: number
		updated: number
		created: number
	}>
}

type JobStatusProps = {
	open: boolean
	onClose: () => void
}

export function JobStatus({ open, onClose }: JobStatusProps) {
	const { lastJsonMessage } = useWebSocket<WebSocketEvent>(`wss://backend.mcjars.app/api/jobs/ws`, {
		retryOnError: true,
		shouldReconnect: () => true
	})

	return (
		<Dialog open={open} onOpenChange={(open) => !open ? onClose() : undefined}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>MCJars Job Status</DialogTitle>
					<DialogDescription>
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Type</TableHead>
									<TableHead>Checked</TableHead>
									<TableHead>Updated</TableHead>
									<TableHead>Created</TableHead>
									<TableHead>Status</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{lastJsonMessage ? (
									Object.entries(lastJsonMessage.jobs).map(([type, status]) => (
										<TableRow key={type}>
											<TableCell className={'flex flex-row'}>
												<img src={`https://s3.mcjars.app/icons/${type.toLowerCase()}.png`} className={'w-6 h-6 hidden md:inline'} />
												<p className={'font-bold md:ml-2'}>{type}</p>
											</TableCell>
											<TableCell>{status.checked}</TableCell>
											<TableCell>{status.updated}</TableCell>
											<TableCell>{status.created}</TableCell>
											<TableCell>
												<Badge className={status.running ? 'bg-green-400 hover:bg-green-300' : 'bg-red-400 hover:bg-red-300'}>{status.running ? 'Running' : 'Stopped'}</Badge>
											</TableCell>
										</TableRow>
									))
								) : (
									<TableRow>
										<TableCell colSpan={5}>Loading...</TableCell>
									</TableRow>
								)}
							</TableBody>
						</Table>
					</DialogDescription>
				</DialogHeader>
			</DialogContent>
		</Dialog>
	)
}