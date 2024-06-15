import { Button } from "@/components/ui/button"
import { useEffect, useState } from "react"
import { TbX } from "react-icons/tb"

export default function Ad() {
	const [ hidden, setHidden ] = useState<boolean>(localStorage.getItem('ad_hidden') === 'true')

	useEffect(() => {
		if (hidden) {
			localStorage.setItem('ad_hidden', 'true')
		} else {
			localStorage.removeItem('ad_hidden')
		}
	}, [ hidden ])

	return (
		<div className={`transition-all absolute justify-between ${!hidden ? 'md:flex' : ''} flex-row hidden w-72 h-36 p-4 right-4 bg-cover bg-[url(/versionchanger_banner.jpg)] bottom-4 backdrop-blur-md rounded-lg`}>
			<a className={'h-fit cursor-pointer hover:text-blue-300 transition-all'} href={'https://www.sourcexchange.net/products/version-changer'} target={'_blank'} rel={'noreferrer'}>
				for Pterodactyl.
			</a>
			<Button size={'icon'} onClick={() => setHidden(true)}>
				<TbX size={24} />
			</Button>
		</div>
	)
}