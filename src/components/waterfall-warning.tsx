import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { useState } from "react"

type WaterfallWarningProps = {
	open: boolean
	onGoToVelocity: () => void
    onGoToBungeeCord: () => void
}

export function WaterfallWarning({ open, onGoToVelocity, onGoToBungeeCord }: WaterfallWarningProps) {
	const [ ignore, setIgnore ] = useState(false)

	return (
		<Dialog open={open && !ignore} onOpenChange={(open) => !open ? setIgnore(true) : undefined}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Are you absolutely sure?</DialogTitle>
					<DialogDescription>
                        Waterfall is <p className={'inline font-bold'}>DEPRECATED</p> and no longer supported,
                        instead you should consider an alternative like <p className={'inline hover:underline cursor-pointer text-blue-500'} onClick={onGoToBungeeCord}>BungeeCord</p> or <p className={'inline hover:underline cursor-pointer text-blue-500'} onClick={onGoToVelocity}>Velocity</p>.
					</DialogDescription>
				</DialogHeader>
			</DialogContent>
		</Dialog>
	)
}