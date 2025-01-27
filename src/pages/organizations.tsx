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
import { ArchiveIcon, CheckIcon, ChevronDown, CodeIcon, FlagIcon, Globe2Icon, GlobeIcon, LinkIcon, LoaderCircle, PlusIcon, TrashIcon, UsersIcon, WebhookIcon, XIcon } from "lucide-react"
import React, { useRef, useState } from "react"
import useSWR from "swr"
import apiGetUserOrganizationApiKeys from "@/api/user/organization/api-keys/apiKeys"
import apiAddUserOrganizationApiKey from "@/api/user/organization/api-keys/addApiKey"
import apiDeleteUserOrganizationApiKey from "@/api/user/organization/api-keys/deleteApiKey"
import { Dialog, DialogContent } from "@/components/ui/dialog"
import { cn } from "@/lib/utils"
import apiPostUserOrganizationIcon from "@/api/user/organization/icon"
import { Badge } from "@/components/ui/badge"
import apiPostUserIniteAccept from "@/api/user/invite/accept"
import apiPostUserIniteDecline from "@/api/user/invite/decline"
import UserTooltip from "@/components/user-tooltip"
import clsx from "clsx"

type OrganizationRowProps = {
	organization: Organization
	currentOrganization: Organization | null
	setCurrentOrganization: React.Dispatch<React.SetStateAction<Organization | null>>
	updateIcon: (url: string) => void
	mutate: () => void
	isPending?: boolean
}

function OrganizationRow({ organization, currentOrganization, setCurrentOrganization, updateIcon, isPending, mutate }: OrganizationRowProps) {
	const [view, setView] = useState<'subusers' | 'api-keys'>()
	const [loading, setLoading] = useState(false)
	const [user, setUser] = useState('')
	const [name, setName] = useState('')
	const [key, setKey] = useState('')
	const [me] = useAuth()
	const inputRef = useRef<HTMLInputElement>(null)

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
			<input type={'file'} accept={'image/*'} onChange={(e) => {
				const t = toast({
					title: 'Updating Organization Icon...',
					description: `Updating the icon for ${organization.name}.`
				})

				apiPostUserOrganizationIcon(organization.id, e.target.files?.[0]!)
					.then((url) => {
						t.update(toast({
							title: 'Organization Icon Updated',
							description: `The icon for ${organization.name} has been updated.`
						}))

						updateIcon(url)
					})
					.catch((error) => {
						t.update(toastError({
							title: 'Failed to Update Organization Icon',
							error
						}))
					})
			}} ref={inputRef} hidden />

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
							<img src={organization.icon ?? ''} alt={'Logo'} className={cn('h-12 w-12 rounded-lg', currentOrganization?.id === organization.id && 'hover:opacity-85 cursor-pointer')} onClick={() => currentOrganization?.id === organization.id && inputRef.current?.click()} />
							<div className={'flex flex-col ml-2'}>
								<h1 className={'text-xl font-semibold flex md:flex-row flex-col md:items-center items-start'}>
									{organization.name}
								</h1>
								<p className={'text-sm text-gray-500 flex flex-col md:flex-row md:items-center items-start'}>
									{new Date(organization.created).toLocaleDateString()}
									<UserTooltip user={organization.owner} className={'md:ml-2'}>
										<span className={'text-blue-400 cursor-pointer'}>@{organization.owner.login}</span>
									</UserTooltip>
								</p>
							</div>
						</div>

						{!isPending ? (
							<div className={'flex flex-row items-center'}>
								<CollapsibleTrigger>
									<Button>
										View
										<ChevronDown size={16} className={'ml-2 transition-transform group-data-[state=open]/collapsible-build:rotate-180'} />
									</Button>
								</CollapsibleTrigger>
							</div>
						) : (
							<div className={'flex flex-row items-center'}>
								<Button disabled={loading} onClick={() => {
									setLoading(true)

									const t = toast({
										title: 'Accepting Organization Invite...',
										description: `Accepting invite to ${organization.name}.`
									})

									apiPostUserIniteAccept(organization.id)
										.then(() => {
											t.update(toast({
												title: 'Organization Invite Accepted',
												description: `You have accepted the invite to ${organization.name}.`
											}))

											mutate()
										})
										.catch((error) => {
											t.update(toastError({
												title: 'Failed to Accept Organization Invite',
												error
											}))
										})
										.finally(() => setLoading(false))
								}}>
									<CheckIcon className={'w-6 h-6 md:mr-2'} />
									<span className={'hidden md:inline'}>Accept</span>
								</Button>
								<Button disabled={loading} variant={'destructive'} className={'ml-2'} onClick={() => {
									setLoading(true)

									const t = toast({
										title: 'Declining Organization Invite...',
										description: `Declining invite to ${organization.name}.`
									})

									apiPostUserIniteDecline(organization.id)
										.then(() => {
											t.update(toast({
												title: 'Organization Invite Declined',
												description: `You have declined the invite to ${organization.name}.`
											}))

											mutate()
										})
										.catch((error) => {
											t.update(toastError({
												title: 'Failed to Decline Organization Invite',
												error
											}))
										})
										.finally(() => setLoading(false))
								}}>
									<XIcon className={'w-6 h-6 md:mr-2'} />
									<span className={'hidden md:inline'}>Decline</span>
								</Button>
							</div>
						)}
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

								const t = toast({
									title: 'Adding Subuser...',
									description: `Adding @${user} to ${organization.name}.`
								})

								apiAddUserOrganizationSubuser(organization.id, user)
									.then(() => {
										t.update(toast({
											title: 'Subuser Added',
											description: `@${user} has been added to ${organization.name}.`
										}))

										setUser('')
										mutateSubusers()
									})
									.catch((error) => {
										t.update(toastError({
											title: 'Failed to Add Subuser',
											error
										}))
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
								<Card key={subuser.user.id} className={'p-4 mt-2'}>
									<div className={'flex flex-row items-center justify-between'}>
										<div className={'flex flex-row items-center text-left'}>
											<img src={subuser.user.avatar ?? ''} alt={'Logo'} className={'h-12 w-12 rounded-lg'} />
											<div className={'flex flex-col ml-2'}>
												<h1 className={'flex flex-row items-center'}>
													<UserTooltip user={subuser.user}>
														<span className={'text-blue-400 cursor-pointer text-xl font-semibold'}>@{subuser.user.login}</span>
													</UserTooltip>
													{subuser.pending && <Badge className={'ml-2'} variant={'destructive'}>Pending</Badge>}
												</h1>
												<p className={'text-sm text-gray-500'}>
													{new Date(subuser.created).toLocaleDateString()}
												</p>
											</div>
										</div>

										<div className={clsx('flex flex-row items-center', me?.id !== organization.owner.id && 'hidden')}>
											<Button variant={'destructive'} disabled={loading} onClick={() => {
												setLoading(true)

												const t = toast({
													title: 'Deleting Subuser...',
													description: `Deleting ${subuser.user.name} from ${organization.name}.`
												})

												apiDeleteUserOrganizationSubuser(organization.id, subuser.user.login)
													.then(() => {
														t.update(toast({
															title: 'Subuser Deleted',
															description: `${subuser.user.name} has been deleted from ${organization.name}.`
														}))

														if (subuser.user.id === me?.id) {
															window.location.reload()
															return
														}

														mutateSubusers((subusers) => !subusers ? null : subusers.filter((s) => s.user.id !== subuser.user.id), false)
													})
													.catch((error) => {
														t.update(toastError({
															title: 'Failed to Delete Subuser',
															error
														}))
													})
													.finally(() => setLoading(false))
											}}>
												<TrashIcon className={'w-6 h-6 md:mr-2'} />
												<span className={'hidden md:inline'}>Delete</span>
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

								const t = toast({
									title: 'Adding API Key...',
									description: `Adding Key to ${organization.name}.`
								})

								apiAddUserOrganizationApiKey(organization.id, name)
									.then((key) => {
										t.update(toast({
											title: 'API Key Added',
											description: `Key has been added to ${organization.name}.`
										}))

										setKey(key)

										setName('')
										mutateApiKeys()
									})
									.catch((error) => {
										t.update(toastError({
											title: 'Failed to Add API Key',
											error
										}))
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
										<div className={'flex flex-row items-center w-[80%]'}>
											<CodeIcon className={'w-12 h-12'} />
											<div className={'flex flex-col ml-2 w-full'}>
												<h1 className={'text-xl font-semibold flex md:flex-row flex-col md:items-center items-start truncate'}>
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

												const t = toast({
													title: 'Deleting API Key...',
													description: `Deleting ${apiKey.name} from ${organization.name}.`
												})

												apiDeleteUserOrganizationApiKey(organization.id, apiKey.id)
													.then(() => {
														t.update(toast({
															title: 'API Key Deleted',
															description: `${apiKey.name} has been deleted from ${organization.name}.`
														}))

														mutateApiKeys((keys) => !keys ? null : keys.filter((k) => k.id !== apiKey.id), false)
													})
													.catch((error) => {
														t.update(toastError({
															title: 'Failed to Delete API Key',
															error
														}))
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

	const { data: organizations, mutate } = useSWR(
		['organizations'],
		() => apiGetUserOrganizations(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	if (!organizations) return (
		<div className={'w-full mt-8 flex flex-row items-center justify-center'}>
			<LoaderCircle className={'animate-spin'} />
		</div>
	)

	const updateIcon = (organization: number) => {
		return (url: string) => {
			mutate((organizations) => {
				if (!organizations) return

				const org = organizations.owned.find((o) => o.id === organization) ?? organizations.member.find((o) => o.id === organization)
				if (org) org.icon = url

				return { ...organizations }
			})
		}
	}

	return (
		<div className={'w-full pb-2 flex flex-col'}>
			<h1 className={'text-2xl font-semibold'}>Owned Organizations</h1>
			{organizations?.owned.map((organization) => (
				<OrganizationRow key={organization.id} mutate={mutate} updateIcon={updateIcon(organization.id)} organization={organization} currentOrganization={currentOrganization} setCurrentOrganization={setCurrentOrganization} />
			))}
			{!organizations?.owned.length && (
				<p className={'text-gray-400 text-sm'}>You do not own any organizations.</p>
			)}

			<h1 className={'text-2xl font-semibold mt-4'}>Member Organizations</h1>
			{organizations?.member.map((organization) => (
				<OrganizationRow key={organization.id} mutate={mutate} updateIcon={updateIcon(organization.id)} organization={organization} currentOrganization={currentOrganization} setCurrentOrganization={setCurrentOrganization} />
			))}
			{!organizations?.member.length && (
				<p className={'text-gray-400 text-sm'}>You are not a member of any organizations.</p>
			)}

			<h1 className={'text-2xl font-semibold mt-4'}>Organization Invites</h1>
			{organizations?.invites.map((organization) => (
				<OrganizationRow key={organization.id} mutate={mutate} updateIcon={updateIcon(organization.id)} organization={organization} currentOrganization={currentOrganization} setCurrentOrganization={setCurrentOrganization} isPending />
			))}
			{!organizations?.invites.length && (
				<p className={'text-gray-400 text-sm'}>You do not have any organization invites.</p>
			)}
		</div>
	)
}