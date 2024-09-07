import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { useState } from "react"

type SpigotWarningProps = {
	open: boolean
	onGoToPaper: () => void
}

export function SpigotWarning({ open, onGoToPaper }: SpigotWarningProps) {
	const [ ignore, setIgnore ] = useState(false)

	return (
		<Dialog open={open && !ignore} onOpenChange={(open) => !open ? setIgnore(true) : undefined}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Are you absolutely sure?</DialogTitle>
					<DialogDescription>
						Spigot is generally <p className={'inline font-bold'}>NOT RECOMMENDED</p> for server use anymore,
						instead you should consider an alternative like <p className={'inline hover:underline cursor-pointer text-blue-500'} onClick={onGoToPaper}>Paper</p>.

						<br />
						<br />

						Spigot generally has more issues and is less optimized than Paper, you should only use Spigot if you have a specific reason to do so.
						{' '}<p className={'inline hover:underline cursor-pointer text-blue-500'} onClick={onGoToPaper}>Paper</p> has more security patches and supports all spigot plugins.
					</DialogDescription>
				</DialogHeader>
			</DialogContent>
		</Dialog>
	)
}