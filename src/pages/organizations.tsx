import apiAddUserOrganizationSubuser from "@/api/user/organization/subusers/addSubuser"
import apiDeleteUserOrganizationSubuser from "@/api/user/organization/subusers/deleteSubuser"
import apiGetUserOrganizationStats from "@/api/user/organization/stats"
import apiGetUserOrganizationSubusers from "@/api/user/organization/subusers/subusers"
import apiGetUserOrganizations, { Organization } from "@/api/user/organizations"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from "@/components/ui/drawer"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { useAuth } from "@/hooks/use-auth"
import { useToast } from "@/hooks/use-toast"
import { ArchiveIcon, ChevronDown, CodeIcon, FlagIcon, Globe2Icon, GlobeIcon, LinkIcon, LoaderCircle, PlusIcon, TrashIcon, UsersIcon, WebhookIcon } from "lucide-react"
import React, { useState } from "react"
import useSWR from "swr"
import apiGetUserOrganizationApiKeys from "@/api/user/organization/api-keys/apiKeys"
import apiAddUserOrganizationApiKey from "@/api/user/organization/api-keys/addApiKey"
import apiDeleteUserOrganizationApiKey from "@/api/user/organization/api-keys/deleteApiKey"
import { Dialog, DialogContent } from "@/components/ui/dialog"

type OrganizationRowProps = {
	organization: Organization
	currentOrganization: Organization | null
	setCurrentOrganization: React.Dispatch<React.SetStateAction<Organization | null>>
}

function OrganizationRow({ organization, currentOrganization, setCurrentOrganization }: OrganizationRowProps) {
	const [view, setView] = useState<'subusers' | 'api-keys'>()
	const [loading, setLoading] = useState(false)
	const [user, setUser] = useState('')
	const [name, setName] = useState('')
	const [key, setKey] = useState('')
	const [me] = useAuth()

	const { toast, toastError } = useToast()

	const { data: stats } = useSWR(
		['organization', organization.id, 'stats', currentOrganization?.id],
		() => currentOrganization?.id === organization.id ? apiGetUserOrganizationStats(organization.id) : null,
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: subUsers, mutate: mutateSubusers } = useSWR(
		['organization', organization.id, 'users', currentOrganization?.id],
		() => currentOrganization?.id === organization.id ? apiGetUserOrganizationSubusers(organization.id) : null,
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: apiKeys, mutate: mutateApiKeys } = useSWR(
		['organization', organization.id, 'apiKeys', currentOrganization?.id],
		() => currentOrganization?.id === organization.id ? apiGetUserOrganizationApiKeys(organization.id) : null,
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	return (
		<>
			<Dialog open={key !== ''} onOpenChange={(open) => !open && setKey('')}>
				<DialogContent className={'md:min-w-[40rem]'}>
					API Key has been created. Please copy it and store it in a safe place.
					<code className={'text-sm font-mono border text-white p-4 rounded-lg w-fit break-all'}>
						{key}
					</code>
				</DialogContent>
			</Dialog>

			<Card className={'mt-2 p-3 pr-4'}>
				<Collapsible open={currentOrganization?.id === organization.id} className={'group/collapsible-build'} onOpenChange={(open) => setCurrentOrganization(open ? organization : null)}>
					<div className={'flex flex-row items-center justify-between'}>
						<div className={'flex flex-row items-center'}>
							<img src={organization.icon ?? ''} alt={'Logo'} className={'h-12 w-12 rounded-lg'} />
							<div className={'flex flex-col ml-2'}>
								<h1 className={'text-xl font-semibold flex md:flex-row flex-col md:items-center items-start'}>
									{organization.name}
								</h1>
								<p className={'text-sm text-gray-500'}>
									{new Date(organization.created).toLocaleDateString()}
								</p>
							</div>
						</div>

						<div className={'flex flex-row items-center'}>
							<CollapsibleTrigger>
								<Button>
									View
									<ChevronDown size={16} className={'ml-2 transition-transform group-data-[state=open]/collapsible-build:rotate-180'} />
								</Button>
							</CollapsibleTrigger>
						</div>
					</div>

					<CollapsibleContent className={'mt-3'}>
						<div className={'grid gap-2 md:grid-cols-[repeat(auto-fit,minmax(30rem,1fr))] w-full'}>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
								<GlobeIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{stats?.requests ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Total Requests</p>
								</div>
							</Card>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
								<ArchiveIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{stats?.ips ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Unique IPs</p>
								</div>
							</Card>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
								<WebhookIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{stats?.userAgents ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Unique User Agents</p>
								</div>
							</Card>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
								<LinkIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{stats?.origins ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Unique Origins</p>
								</div>
							</Card>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
								<Globe2Icon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{stats?.continents ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Unique Continents</p>
								</div>
							</Card>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
								<FlagIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{stats?.countries ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Unique Countries</p>
								</div>
							</Card>
						</div>
						<div className={'mt-2 grid gap-2 md:grid-cols-[repeat(auto-fit,minmax(30rem,1fr))] w-full'}>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between hover:border-gray-50 cursor-pointer'} onClick={() => setView('subusers')}>
								<UsersIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{subUsers?.length ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>Subusers</p>
								</div>
							</Card>
							<Card className={'p-4 min-w-40 flex flex-row items-center justify-between hover:border-gray-50 cursor-pointer'} onClick={() => setView('api-keys')}>
								<CodeIcon className={'w-8 h-8'} />

								<div className={'flex flex-col text-right items-end'}>
									<h1 className={'text-xl font-semibold'}>
										{apiKeys?.length ?? <Skeleton className={'w-20 h-7'} />}
									</h1>
									<p className={'text-sm text-muted-foreground'}>API Keys</p>
								</div>
							</Card>
						</div>
					</CollapsibleContent>
				</Collapsible>
			</Card>

			<Drawer open={view === 'subusers'} onClose={() => setView(undefined)} setBackgroundColorOnScale={false}>
				<DrawerContent className={'w-full max-w-5xl mx-auto'} onPointerDownOutside={() => setView(undefined)}>
					<DrawerHeader>
						<DrawerTitle className={'flex flex-row justify-between items-center'}>
							Subusers ({subUsers?.length ?? 0})

							<form className={'flex flex-row items-center'} onSubmit={(e) => {
								e.preventDefault()

								setLoading(true)

								toast({
									title: 'Adding Subuser...',
									description: `Adding @${user} to ${organization.name}.`
								})

								apiAddUserOrganizationSubuser(organization.id, user)
									.then(() => {
										toast({
											title: 'Subuser Added',
											description: `@${user} has been added to ${organization.name}.`
										})

										setUser('')
										mutateSubusers()
									})
									.catch((error) => {
										toastError({
											title: 'Failed to Add Subuser',
											error
										})
									})
									.finally(() => setLoading(false))
							}}>
								<Input placeholder={'User Handle (@user)'} className={'mr-2'} value={user} onChange={(e) => setUser(e.target.value)} disabled={loading} />
								<Button disabled={loading || !user} type={'submit'}>
									<PlusIcon className={'w-6 h-6 mr-2'} />
									Add User
								</Button>
							</form>
						</DrawerTitle>
					</DrawerHeader>

					{!subUsers ? (
						<div className={'flex flex-col items-center justify-center'}>
							<LoaderCircle className={'animate-spin'} />
						</div>
					) : (
						<div className={'flex flex-col p-4'}>
							{subUsers.map((subuser) => (
								<Card key={subuser.id} className={'p-4 mt-2'}>
									<div className={'flex flex-row items-center justify-between'}>
										<div className={'flex flex-row items-center'}>
											<img src={subuser.avatar ?? ''} alt={'Logo'} className={'h-12 w-12 rounded-lg'} />
											<div className={'flex flex-col ml-2'}>
												<h1 className={'text-xl font-semibold flex md:flex-row flex-col md:items-center items-start'}>
													{subuser.name}
												</h1>
												<p className={'text-sm text-gray-500'}>
													@{subuser.login}
												</p>
											</div>
										</div>

										<div className={'flex flex-row items-center'}>
											<Button variant={'destructive'} disabled={loading} onClick={() => {
												setLoading(true)

												toast({
													title: 'Deleting Subuser...',
													description: `Deleting ${subuser.name} from ${organization.name}.`
												})

												apiDeleteUserOrganizationSubuser(organization.id, subuser.login)
													.then(() => {
														toast({
															title: 'Subuser Deleted',
															description: `${subuser.name} has been deleted from ${organization.name}.`
														})

														if (subuser.id === me?.id) {
															window.location.reload()
															return
														}

														mutateSubusers((subusers) => !subusers ? null : subusers.filter((s) => s.id !== subuser.id), false)
													})
													.catch((error) => {
														toastError({
															title: 'Failed to Delete Subuser',
															error
														})
													})
													.finally(() => setLoading(false))
											}}>
												<TrashIcon className={'w-6 h-6 mr-2'} />
												Delete
											</Button>
										</div>
									</div>
								</Card>
							))}
							{!subUsers.length && (
								<p className={'text-gray-400 text-sm'}>There are no subusers.</p>
							)}
						</div>
					)}
				</DrawerContent>
			</Drawer>

			<Drawer open={view === 'api-keys'} onClose={() => setView(undefined)} setBackgroundColorOnScale={false}>
				<DrawerContent className={'w-full max-w-5xl mx-auto'} onPointerDownOutside={() => setView(undefined)}>
					<DrawerHeader>
						<DrawerTitle className={'flex flex-row justify-between items-center'}>
							API Keys ({apiKeys?.length ?? 0})

							<form className={'flex flex-row items-center'} onSubmit={(e) => {
								e.preventDefault()

								setLoading(true)

								toast({
									title: 'Adding API Key...',
									description: `Adding Key to ${organization.name}.`
								})

								apiAddUserOrganizationApiKey(organization.id, name)
									.then((key) => {
										toast({
											title: 'API Key Added',
											description: `Key has been added to ${organization.name}.`
										})

										setKey(key)

										setName('')
										mutateApiKeys()
									})
									.catch((error) => {
										toastError({
											title: 'Failed to Add API Key',
											error
										})
									})
									.finally(() => setLoading(false))
							}}>
								<Input placeholder={'Key Name'} className={'mr-2'} value={name} onChange={(e) => setName(e.target.value)} disabled={loading} />
								<Button disabled={loading || !name} type={'submit'}>
									<PlusIcon className={'w-6 h-6 mr-2'} />
									Add Key
								</Button>
							</form>
						</DrawerTitle>
					</DrawerHeader>

					{!apiKeys ? (
						<div className={'flex flex-col items-center justify-center'}>
							<LoaderCircle className={'animate-spin'} />
						</div>
					) : (
						<div className={'flex flex-col p-4'}>
							{apiKeys.map((apiKey) => (
								<Card key={apiKey.id} className={'p-4 mt-2'}>
									<div className={'flex flex-row items-center justify-between'}>
										<div className={'flex flex-row items-center'}>
											<CodeIcon className={'w-12 h-12'} />
											<div className={'flex flex-col ml-2'}>
												<h1 className={'text-xl font-semibold flex md:flex-row flex-col md:items-center items-start'}>
													{apiKey.name}
												</h1>
												<p className={'text-sm text-gray-500'}>
													{new Date(apiKey.created).toLocaleDateString()}
												</p>
											</div>
										</div>

										<div className={'flex flex-row items-center'}>
											<Button variant={'destructive'} disabled={loading} onClick={() => {
												setLoading(true)

												toast({
													title: 'Deleting API Key...',
													description: `Deleting ${apiKey.name} from ${organization.name}.`
												})

												apiDeleteUserOrganizationApiKey(organization.id, apiKey.id)
													.then(() => {
														toast({
															title: 'API Key Deleted',
															description: `${apiKey.name} has been deleted from ${organization.name}.`
														})

														mutateApiKeys((keys) => !keys ? null : keys.filter((k) => k.id !== apiKey.id), false)
													})
													.catch((error) => {
														toastError({
															title: 'Failed to Delete API Key',
															error
														})
													})
													.finally(() => setLoading(false))
											}}>
												<TrashIcon className={'w-6 h-6 mr-2'} />
												Delete
											</Button>
										</div>
									</div>
								</Card>
							))}
							{!apiKeys.length && (
								<p className={'text-gray-400 text-sm'}>There are no api keys.</p>
							)}
						</div>
					)}
				</DrawerContent>
			</Drawer>
		</>
	)
}

export default function PageOrganizations() {
	const [currentOrganization, setCurrentOrganization] = useState<Organization | null>(null)

	const { data: organizations } = useSWR(
		['organizations'],
		() => apiGetUserOrganizations(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	if (!organizations) return (
		<div className={'w-full mt-8 flex flex-row items-center justify-center'}>
			<LoaderCircle className={'animate-spin'} />
		</div>
	)

	return (
		<>
			<h1 className={'text-2xl font-semibold'}>Owned Organizations</h1>
			{organizations?.owned.map((organization) => (
				<OrganizationRow key={organization.id} organization={organization} currentOrganization={currentOrganization} setCurrentOrganization={setCurrentOrganization} />
			))}
			{!organizations?.owned.length && (
				<p className={'text-gray-400 text-sm'}>You do not own any organizations.</p>
			)}

			<h1 className={'text-2xl font-semibold mt-4'}>Member Organizations</h1>
			{organizations?.member.map((organization) => (
				<OrganizationRow key={organization.id} organization={organization} currentOrganization={currentOrganization} setCurrentOrganization={setCurrentOrganization} />
			))}
			{!organizations?.member.length && (
				<p className={'text-gray-400 text-sm'}>You are not a member of any organizations.</p>
			)}
		</>
	)
}