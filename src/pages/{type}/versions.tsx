import apiGetBuilds from "@/api/builds"
import apiGetTypes from "@/api/types"
import apiGetVersions from "@/api/versions"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from "@/components/ui/drawer"
import { Input } from "@/components/ui/input"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { useIsMobile } from "@/hooks/use-mobile"
import bytes from "bytes"
import { ChevronDown, DownloadIcon, ListIcon, SearchIcon } from "lucide-react"
import { useEffect, useMemo } from "react"
import { Link, useParams } from "react-router-dom"
import useSWR from "swr"
import { StringParam, useQueryParam } from "use-query-params"

export default function PageTypeVersions() {
	const { type } = useParams<{ type: string }>()
	if (!type) return null

	const mobile = useIsMobile(1280)

	const [ versionType, setVersionType ] = useQueryParam('type', StringParam)
	const [ search, setSearch ] = useQueryParam('search', StringParam)
	const [ browse, setBrowse ] = useQueryParam('browse', StringParam)
	const [ displayMode, setDisplayMode ] = useQueryParam('display', StringParam)

	useEffect(() => {
		if (mobile) {
			setDisplayMode('list')
		}
	}, [ mobile ])

	const { data: types } = useSWR(
		['types'],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: versions } = useSWR(
		['versions', type],
		() => apiGetVersions(type),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: builds } = useSWR(
		['builds', type, browse],
		() => browse ? apiGetBuilds(type, browse) : undefined,
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const filteredVersions = versions?.filter((version) => versionType === 'all' || !versionType || (versionType === 'stable' ? version.type === 'RELEASE' : version.type === 'SNAPSHOT'))
		.filter((version) => !search || version.latest.versionId?.toLowerCase().includes(search.toLowerCase()))

	const expectedBuildCount = useMemo(
		() => !browse ? 0 : versions?.find((version) => version.latest.versionId === browse)?.builds ?? 0,
		[ versions, browse ]
	)

	return (
		<>
			{(types?.find((t) => t.identifier === type)?.experimental || types?.find((t) => t.identifier === type)?.deprecated) && (
				<Alert className={'mb-2'} variant={'destructive'}>
					<AlertDescription>
						Keep in mind, <span className={'font-semibold'}>{types?.find((t) => t.identifier === type)?.name}</span> is {types?.find((t) => t.identifier === type)?.experimental ? 'experimental' : 'deprecated'} and may not work as expected. Take backups!
					</AlertDescription>
				</Alert>
			)}

			{type === 'SPIGOT' && (
				<Alert className={'mb-2'}>
					<AlertDescription>
						Hello there, <span className={'font-semibold'}>Spigot</span> is not recommended for server-use due to its worse performance and security compared to other server software. Please consider using <Link to={'/PAPER/versions'} className={'text-blue-500 font-semibold underline'}>Paper</Link> instead.
					</AlertDescription>
				</Alert>
			)}

			<div className={'flex flex-row items-center mb-6 md:w-1/2 w-full'}>
				<Select value={versionType ?? 'all'} onValueChange={(value) => setVersionType(value)}>
					<SelectTrigger className={'w-[15em]'}>
						<SelectValue placeholder={'All Versions'} />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={'stable'}>Stable Versions</SelectItem>
						<SelectItem value={'snapshot'}>Snapshot Versions</SelectItem>
						<SelectItem value={'all'}>All Versions</SelectItem>
					</SelectContent>
				</Select>

				<Select value={displayMode ?? 'compact'} onValueChange={(value) => setDisplayMode(value)} disabled={mobile}>
					<SelectTrigger className={'w-[11em] ml-2'}>
						<SelectValue placeholder={'Compact'} />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={'grid'}>Grid</SelectItem>
						<SelectItem value={'list'}>List</SelectItem>
						<SelectItem value={'compact'}>Compact</SelectItem>
					</SelectContent>
				</Select>

				<Input className={'ml-2'} placeholder={'Search Name'} value={search ?? ''} onChange={(e) => setSearch(e.target.value)} />
			</div>

			<div className={displayMode === 'grid' ? 'grid md:grid-cols-2 grid-cols-1' : displayMode === 'list' ? 'flex flex-col' : 'grid grid-cols-[repeat(auto-fit,minmax(30rem,1fr))]'}>
				{!filteredVersions ? (
					<>
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 lg:min-w-[30rem]'} />
					</>
				) : (
					<>
						{filteredVersions.map((version) => (
							<Card key={version.latest.versionId ?? version.latest.projectVersionId} className={'mb-2 mx-1 p-3 pr-4 xl:min-w-[30rem] flex-1'}>
								<div className={'flex flex-row items-center justify-between'}>
									<div className={'flex flex-row items-center'}>
										<img src={`https://s3.mcjars.app/icons/${type.toLowerCase()}.png`} alt={'Logo'} className={'h-12 w-12 rounded-md'} />
										<div className={'flex flex-col ml-2'}>
											<h1 className={'text-xl font-semibold flex flex-row items-center'}>
												{version.latest.versionId ?? version.latest.projectVersionId}
												<Badge className={'ml-2'} variant={version.type === 'RELEASE' ? 'outline' : 'destructive'}>
													{version.type}
												</Badge>
											</h1>
											<p className={'text-sm text-gray-500'}>
												{version.builds} Build{version.builds === 1 ? '' : 's'}, Java {version.java}, {new Date(version.created).toLocaleDateString()}
											</p>
										</div>
									</div>

									<div className={'flex flex-row items-center'}>
										<Button variant={'outline'} onClick={() => setBrowse(version.latest.versionId ?? version.latest.projectVersionId)} disabled={Boolean(browse)}>
											<SearchIcon size={16} className={'mr-2'} />
											Browse
										</Button>
									</div>
								</div>
							</Card>
						))}
						{!filteredVersions.length && (
							<p className={'text-center text-gray-500'}>No versions found.</p>
						)}
					</>
				)}
			</div>

			<Drawer open={Boolean(browse)} onClose={() => setBrowse(undefined)} setBackgroundColorOnScale={false}>
				<DrawerContent className={'w-full max-w-5xl mx-auto'} onPointerDownOutside={() => setBrowse(undefined)}>
					<DrawerHeader>
						<DrawerTitle>Browse {browse}</DrawerTitle>
					</DrawerHeader>
					<div className={'p-4 h-full max-h-96 overflow-y-auto'}>
						{!browse ? (
							<div className={'h-32'} />
						) : !builds ? (
							<>
								<Skeleton className={'h-16 rounded-xl mb-2'} />
								<Skeleton className={'h-16 rounded-xl mb-2'} />
								{expectedBuildCount > 2 && <Skeleton className={'h-16 rounded-xl mb-2'} />}
								{expectedBuildCount > 3 && <Skeleton className={'h-16 rounded-xl mb-2'} />}
								{expectedBuildCount > 4 && <Skeleton className={'h-16 rounded-xl mb-2'} />}
							</>
						) : (
							<>
								{builds.map((build, i) => (
									<Card key={build.id} className={'mb-2 p-3 pr-4'}>
										<Collapsible defaultOpen={i === 0} className={'group/collapsible-build'}>
											<div className={'flex flex-row items-center justify-between'}>
												<div className={'flex flex-row items-center'}>
													<img src={`https://s3.mcjars.app/icons/${type.toLowerCase()}.png`} alt={'Logo'} className={'h-12 w-12'} />
													<div className={'flex flex-col ml-2'}>
														<h1 className={'text-xl font-semibold flex flex-row items-center'}>
															{build.name}
															<Badge className={'ml-2'} variant={build.experimental ? 'destructive' : 'outline'}>
																{build.experimental ? 'Experimental' : 'Stable'}
															</Badge>
														</h1>
														<p className={'text-sm text-gray-500'}>
															{bytes(build.installation.flat().filter((step) => step.type === 'download').reduce((a, c) => a + c.size, 0))}, {build.created ? new Date(build.created).toLocaleDateString() : `${build.changes.length} Change${build.changes.length === 1 ? '' : 's'}`}
														</p>
													</div>
												</div>

												<div className={'flex flex-row items-center'}>
													{build.changes.length > 0 && (
														<Popover modal>
															<PopoverTrigger>
																<Button variant={'outline'} className={'mr-2 hidden md:flex'}>
																	<ListIcon size={16} className={'mr-2'} />
																	Changes
																</Button>
															</PopoverTrigger>
															<PopoverContent align={'start'} className={'max-h-32 overflow-y-scroll'}>
																<div className={'flex flex-col'}>
																	{build.changes.map((c, i) => (
																		<p key={i} className={'text-xs'}>- {c}</p>
																	))}
																</div>
															</PopoverContent>
														</Popover>
													)}

													<CollapsibleTrigger>
														<Button>
															Install
															<ChevronDown size={16} className={'ml-2 transition-transform group-data-[state=open]/collapsible-build:rotate-180'} />
														</Button>
													</CollapsibleTrigger>
												</div>
											</div>

											<CollapsibleContent className={'mt-2'}>
												{build.installation.flat().map((step, i) => (
													<div key={i} className={'flex flex-row items-center mt-2'}>
														{step.type[0].toUpperCase() + step.type.slice(1)}

														<div className={'mx-1'} />

														{step.type === 'download' && (
															<a href={step.url} download>
																<Button className={'w-fit'} variant={'outline'}>
																	<DownloadIcon size={16} className={'mr-2'} />
																	{step.file}
																</Button>
															</a>
														)}
														{step.type === 'remove' && (
															<code className={'border rounded-md p-1 px-3 w-fit'}>
																{step.location}
															</code>
														)}
														{step.type === 'unzip' && (
															<div className={'flex flex-row items-center'}>
																<code className={'border rounded-md p-1 px-3 w-fit'}>
																	{step.file}
																</code>

																<p className={'mx-2'}>to</p>

																<code className={'border rounded-md p-1 px-3 w-fit'}>
																	{step.location}
																</code>
															</div>
														)}
													</div>
												))}
											</CollapsibleContent>
										</Collapsible>
									</Card>
								))}
							</>
						)}
					</div>
				</DrawerContent>
			</Drawer>
		</>
	)
}