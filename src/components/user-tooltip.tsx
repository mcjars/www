import { User } from "@/api/user/infos"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import { ExternalLinkIcon } from "lucide-react"

export default function UserTooltip({ user, children, className }: { user: User, children?: React.ReactNode, className?: string }) {
	return (
		<Tooltip>
			<TooltipContent className={'flex flex-row items-center'}>
				<img src={user.avatar ?? ''} alt={'Owner'} className={'h-12 w-12 rounded-lg'} />
				<div className={'flex flex-col ml-1.5 text-left'}>
					<a className={'text-lg flex flex-row items-center hover:underline cursor-pointer'} href={`https://github.com/${user.login}`} target={'_blank'} rel={'noreferrer'}>
						{user.name ?? user.login}
						<ExternalLinkIcon size={16} className={'ml-1.5'} />
					</a>
					<p className={'text-sm text-gray-500'}>{user.email}</p>
				</div>
			</TooltipContent>
			<TooltipTrigger className={className}>
				{children}
			</TooltipTrigger>
		</Tooltip>
	)
}