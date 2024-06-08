import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"

type FoliaFlowchartProps = {
	open: boolean
	onClose: () => void
}

type Question = {
	text: string
	answers: string[]

	answer: number
}

const questions: Question[] = [
	{
		text: 'Do you expect to use your Paper/Spigot plugins?',
		answers: [
			'Yes', 'No'
		], answer: 1
	},
	{
		text: 'Do you have 8+ cores assigned to your server?',
		answers: [
			'Yes', 'No'
		], answer: 0
	},
	{
		text: 'Do you agree that issues with plugins are not papers responsibility?',
		answers: [
			'Yes', 'No'
		], answer: 0
	}
]

export function FoliaFlowchart({ open, onClose }: FoliaFlowchartProps) {
	const [ step, setStep ] = useState(0)
	const [ done, setDone ] = useState(false)
	const [ answers, setAnswers ] = useState<number[]>([])

	useEffect(() => {
		if (answers.length === questions.length && answers.every((answer, index) => answer === questions[index].answer)) {
			console.log('You are ready for Folia!')
			setDone(true)
		}
	}, [answers])

	return (
		<Dialog open={open && !done} onOpenChange={(open) => !open ? onClose() : undefined}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Are you absolutely sure?</DialogTitle>
					<DialogDescription>
						{step !== questions.length
							? <>Folia is not just another paper fork, it is highly experimental. To continue, you must answer the following questions:</>
							: <>Folia is not just another paper fork, it is highly experimental.</>
						}

						{step !== questions.length ? (
							<div className={'mt-4 w-full flex-col'}>
								<h1 className={'text-white font-semibold text-xl'}>{questions[step].text}</h1>
								<div className={'mt-1 w-full grid grid-cols-2 gap-2'}>
									{questions[step].answers.map((answer, index) => (
										<Button onClick={() => {
											setStep((step) => step + 1)
											setAnswers((answers) => [ ...answers, index ])
										}}>
											{answer}
										</Button>
									))}
								</div>
							</div>
						) : (
							<div className={'mt-4 w-full flex-col'}>
								<h1 className={'text-white font-semibold text-xl'}>You are not ready for Folia!</h1>
							</div>
						)}
					</DialogDescription>
				</DialogHeader>
			</DialogContent>
		</Dialog>
	)
}