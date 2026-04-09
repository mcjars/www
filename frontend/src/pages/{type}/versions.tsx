import apiGetBuilds from "@/api/builds"
import apiGetTypes from "@/api/types"
import apiGetVersions from "@/api/versions"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { ResponsiveTooltip, Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from "@/components/ui/drawer"
import { Input } from "@/components/ui/input"
import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "@/components/ui/pagination"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { useIsMobile } from "@/hooks/use-mobile"
import bytes from "bytes"
import { ChevronDown, DownloadIcon, ExternalLinkIcon, ListIcon, RefreshCwIcon, SearchIcon, TriangleAlertIcon } from "lucide-react"
import { useEffect, useMemo, useRef, useState } from "react"
import { useNavigate, useParams } from "react-router-dom"
import useSWR from "swr"
import { NumberParam, StringParam, useQueryParam } from "use-query-params"
import { useLocalStorage } from "usehooks-ts"

export default function PageTypeVersions() {
	const { type } = useParams<{ type: string }>()
	if (!type) return null
	const navigate = useNavigate()

	const mobile = useIsMobile(1280)

	const [versionType, setVersionType] = useQueryParam('type', StringParam)
	const [search, setSearch] = useQueryParam('search', StringParam)
	const [browse, setBrowse] = useQueryParam('browse', StringParam)
	const [versionPage, setVersionPage] = useQueryParam('page', NumberParam)
	const [buildSearch, setBuildSearch] = useQueryParam('buildSearch', StringParam)
	const [buildPage, setBuildPage] = useQueryParam('buildPage', NumberParam)
	const [displayMode, setDisplayMode] = useQueryParam('display', StringParam)
	const [installScript, setInstallScript] = useLocalStorage<'bash' | 'mcvcli'>('install-script', 'bash')
	const [versionsPerPage, setVersionsPerPage] = useLocalStorage<number>('versions-per-page', 24)
	const [hasCustomVersionsPerPage, setHasCustomVersionsPerPage] = useLocalStorage<boolean>('versions-per-page-customized', false)
	const [buildsPerPage, setBuildsPerPage] = useLocalStorage<number>('builds-per-page', 20)
	const [compactColumns, setCompactColumns] = useState(1)
	const versionsContainerRef = useRef<HTMLDivElement>(null)

	const currentVersionPage = Math.max(versionPage ?? 1, 1)
	const currentBuildPage = Math.max(buildPage ?? 1, 1)
	const effectiveDisplayMode = displayMode ?? (mobile ? 'list' : 'compact')

	const { data: types } = useSWR(
		['types'],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: versionsResponse } = useSWR(
		['versions', type, currentVersionPage, search ?? '', versionsPerPage],
		() => apiGetVersions(type, {
			page: currentVersionPage,
			perPage: versionsPerPage,
			search: search ?? ''
		}),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: buildsResponse } = useSWR(
		['builds', type, browse, currentBuildPage, buildSearch ?? '', buildsPerPage],
		() => browse
			? apiGetBuilds(type, browse, {
				page: currentBuildPage,
				perPage: buildsPerPage,
				search: buildSearch ?? ''
			})
			: undefined,
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const versions = versionsResponse?.items
	const builds = buildsResponse?.items

	const filteredVersions = versions?.filter((version) => versionType === 'all' || !versionType || (versionType === 'stable' ? version.type === 'RELEASE' : version.type === 'SNAPSHOT'))

	const expectedBuildCount = useMemo(
		() => !browse ? 0 : buildsResponse?.total ?? versions?.find((version) => version.latest.versionId === browse)?.builds ?? 0,
		[buildsResponse, versions, browse]
	)

	useEffect(() => {
		setBuildPage(1)
		setBuildSearch('')
	}, [browse])

	useEffect(() => {
		const updateColumns = () => {
			if (effectiveDisplayMode !== 'compact' || mobile) {
				setCompactColumns(1)
				return
			}

			const width = versionsContainerRef.current?.clientWidth ?? window.innerWidth
			const minCardWidth = 480
			setCompactColumns(Math.max(1, Math.floor(width / minCardWidth)))
		}

		updateColumns()
		window.addEventListener('resize', updateColumns)
		return () => window.removeEventListener('resize', updateColumns)
	}, [effectiveDisplayMode, mobile])

	const recommendedVersionsPerPage = useMemo(() => {
		if (effectiveDisplayMode === 'list') return 24

		const columns = effectiveDisplayMode === 'grid'
			? (mobile ? 1 : 2)
			: Math.max(1, compactColumns)

		const rows = Math.max(4, Math.floor(24 / columns))
		return Math.min(200, Math.max(columns, rows * columns))
	}, [effectiveDisplayMode, mobile, compactColumns])

	const versionPageSizeOptions = useMemo(
		() => Array.from(new Set([12, 16, 20, 24, 32, 40, 50, 100, 200, recommendedVersionsPerPage, versionsPerPage])).sort((a, b) => a - b),
		[recommendedVersionsPerPage, versionsPerPage]
	)

	useEffect(() => {
		if (!hasCustomVersionsPerPage && versionsPerPage !== recommendedVersionsPerPage) {
			setVersionsPerPage(recommendedVersionsPerPage)
		}
	}, [hasCustomVersionsPerPage, recommendedVersionsPerPage, versionsPerPage, setVersionsPerPage])

	useEffect(() => {
		setVersionPage(1)
	}, [versionsPerPage])

	useEffect(() => {
		setBuildPage(1)
	}, [buildsPerPage])

	const typeData = useMemo(
		() => Object.values(types ?? {}).flat()?.find((t) => t.identifier === type),
		[types, type]
	)

	const totalVersionPages = useMemo(
		() => {
			if (!versionsResponse) return 1

			const total = versionsResponse.total
			const responsePage = versionsResponse.page ?? currentVersionPage
			const responsePerPage = versionsResponse.perPage ?? versionsPerPage

			if (typeof total === 'number' && total > 0) {
				return Math.max(1, Math.ceil(total / responsePerPage))
			}

			if (versionsResponse.hasNextPage) {
				return responsePage + 1
			}

			return Math.max(1, responsePage)
		},
		[versionsResponse, versionsPerPage, currentVersionPage]
	)

	const totalBuildPages = useMemo(
		() => {
			if (!buildsResponse) return 1

			const total = buildsResponse.total
			const responsePage = buildsResponse.page ?? currentBuildPage
			const responsePerPage = buildsResponse.perPage ?? buildsPerPage

			if (typeof total === 'number' && total > 0) {
				return Math.max(1, Math.ceil(total / responsePerPage))
			}

			if (buildsResponse.hasNextPage) {
				return responsePage + 1
			}

			return Math.max(1, responsePage)
		},
		[buildsResponse, buildsPerPage, currentBuildPage]
	)

	const getVersionPageNumbers = useMemo(() => {
		const pages: (number | string)[] = []
		const maxVisible = 5
		const total = totalVersionPages

		if (total <= maxVisible) {
			for (let i = 1; i <= total; i++) pages.push(i)
		} else {
			pages.push(1)
			if (currentVersionPage > 3) pages.push('...')
			for (let i = Math.max(2, currentVersionPage - 1); i <= Math.min(total - 1, currentVersionPage + 1); i++) {
				if (!pages.includes(i)) pages.push(i)
			}
			if (currentVersionPage < total - 2) pages.push('...')
			pages.push(total)
		}
		return pages
	}, [totalVersionPages, currentVersionPage])

	const getBuildPageNumbers = useMemo(() => {
		const pages: (number | string)[] = []
		const maxVisible = 5
		const total = totalBuildPages

		if (total <= maxVisible) {
			for (let i = 1; i <= total; i++) pages.push(i)
		} else {
			pages.push(1)
			if (currentBuildPage > 3) pages.push('...')
			for (let i = Math.max(2, currentBuildPage - 1); i <= Math.min(total - 1, currentBuildPage + 1); i++) {
				if (!pages.includes(i)) pages.push(i)
			}
			if (currentBuildPage < total - 2) pages.push('...')
			pages.push(total)
		}
		return pages
	}, [totalBuildPages, currentBuildPage])

	useEffect(() => {
		const isTypingTarget = (target: EventTarget | null) => {
			if (!(target instanceof HTMLElement)) return false
			const tag = target.tagName
			return target.isContentEditable || tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
		}

		const isPrevKey = (event: KeyboardEvent) => event.key === '<' || event.key === 'ArrowLeft'
		const isNextKey = (event: KeyboardEvent) => event.key === '>' || event.key === 'ArrowRight'

		const onKeyDown = (event: KeyboardEvent) => {
			if (event.ctrlKey || event.metaKey || event.altKey) return
			if (!isPrevKey(event) && !isNextKey(event)) return
			if (isTypingTarget(event.target)) return

			if (browse) {
				event.preventDefault()
				if (isPrevKey(event)) {
					setBuildPage(event.shiftKey ? 1 : Math.max(1, currentBuildPage - 1))
					return
				}

				setBuildPage(event.shiftKey ? totalBuildPages : currentBuildPage + 1)
				return
			}

			event.preventDefault()
			if (isPrevKey(event)) {
				setVersionPage(event.shiftKey ? 1 : Math.max(1, currentVersionPage - 1))
				return
			}

			setVersionPage(event.shiftKey ? totalVersionPages : currentVersionPage + 1)
		}

		window.addEventListener('keydown', onKeyDown)
		return () => window.removeEventListener('keydown', onKeyDown)
	}, [
		browse,
		currentBuildPage,
		currentVersionPage,
		totalBuildPages,
		totalVersionPages,
		setBuildPage,
		setVersionPage
	])

	const versionPaginationControls = (
		<div className={'flex flex-row items-center justify-end gap-2'}>
			<Select
				value={hasCustomVersionsPerPage ? String(versionsPerPage) : 'auto'}
				onValueChange={(value) => {
					if (value === 'auto') {
						setHasCustomVersionsPerPage(false)
						setVersionsPerPage(recommendedVersionsPerPage)
						return
					}

					setHasCustomVersionsPerPage(true)
					setVersionsPerPage(Number(value))
				}}
			>
				<SelectTrigger className={'w-44'}>
					<SelectValue placeholder={'Page Size'} />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={'auto'}>Auto ({recommendedVersionsPerPage} / page)</SelectItem>
					{versionPageSizeOptions.map((size) => (
						<SelectItem key={size} value={String(size)}>{size} / page</SelectItem>
					))}
				</SelectContent>
			</Select>

			<Pagination className={'mx-0 w-fit justify-end'}>
				<PaginationContent>
					<PaginationItem>
						<PaginationPrevious
							href="#"
							onClick={(e) => {
								e.preventDefault()
								if (currentVersionPage > 1) setVersionPage(currentVersionPage - 1)
							}}
							className={currentVersionPage <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
						/>
					</PaginationItem>

					{getVersionPageNumbers.map((pageNum, i) => (
						typeof pageNum === 'string' ? (
							<PaginationItem key={`ellipsis-${i}`}>
								<PaginationEllipsis />
							</PaginationItem>
						) : (
							<PaginationItem key={pageNum}>
								<PaginationLink
									href="#"
									isActive={pageNum === currentVersionPage}
									onClick={(e) => {
										e.preventDefault()
										setVersionPage(pageNum as number)
									}}
								>
									{pageNum}
								</PaginationLink>
							</PaginationItem>
						)
					))}

					<PaginationItem>
						<PaginationNext
							href="#"
							onClick={(e) => {
								e.preventDefault()
								if (versionsResponse?.hasNextPage) setVersionPage(currentVersionPage + 1)
							}}
							className={!versionsResponse?.hasNextPage ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
						/>
					</PaginationItem>
				</PaginationContent>
			</Pagination>
		</div>
	)

	const buildPaginationControls = (
		<div className={'flex flex-row items-center justify-end gap-2'}>
			<Select value={String(buildsPerPage)} onValueChange={(value) => setBuildsPerPage(Number(value))}>
				<SelectTrigger className={'w-36'}>
					<SelectValue placeholder={'Page Size'} />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={'10'}>10 / page</SelectItem>
					<SelectItem value={'20'}>20 / page</SelectItem>
					<SelectItem value={'50'}>50 / page</SelectItem>
					<SelectItem value={'100'}>100 / page</SelectItem>
					<SelectItem value={'200'}>200 / page</SelectItem>
				</SelectContent>
			</Select>

			<Pagination className={'mx-0 w-fit justify-end'}>
				<PaginationContent>
					<PaginationItem>
						<PaginationPrevious
							href="#"
							onClick={(e) => {
								e.preventDefault()
								if (currentBuildPage > 1) setBuildPage(currentBuildPage - 1)
							}}
							className={currentBuildPage <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
						/>
					</PaginationItem>

					{getBuildPageNumbers.map((pageNum, i) => (
						typeof pageNum === 'string' ? (
							<PaginationItem key={`ellipsis-${i}`}>
								<PaginationEllipsis />
							</PaginationItem>
						) : (
							<PaginationItem key={pageNum}>
								<PaginationLink
									href="#"
									isActive={pageNum === currentBuildPage}
									onClick={(e) => {
										e.preventDefault()
										setBuildPage(pageNum as number)
									}}
								>
									{pageNum}
								</PaginationLink>
							</PaginationItem>
						)
					))}

					<PaginationItem>
						<PaginationNext
							href="#"
							onClick={(e) => {
								e.preventDefault()
								if (buildsResponse?.hasNextPage) setBuildPage(currentBuildPage + 1)
							}}
							className={!buildsResponse?.hasNextPage ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
						/>
					</PaginationItem>
				</PaginationContent>
			</Pagination>
		</div>
	)

	return (
		<>
			<Alert className={'mb-2'} variant={typeData?.deprecated || typeData?.experimental ? 'destructive' : 'default'}>
				<AlertDescription className={'flex flex-row items-center justify-between'}>
					<div className={'flex flex-row items-center'}>
						<img src={typeData?.icon} alt={'Logo'} className={'h-10 w-10 rounded-md mr-3'} />
						<div className={'flex flex-col justify-center'}>
							<h1 className={'text-2xl font-semibold flex flex-row items-center'}>
								{typeData?.name}
							</h1>
							<span className={'md:block hidden mt-1'}>
								<Badge className={'mr-2'} variant={typeData?.experimental || typeData?.deprecated ? 'destructive' : 'outline'}>
									{typeData?.experimental ? 'Experimental' : typeData?.deprecated ? 'Deprecated' : 'Stable'}
								</Badge>

								{typeData?.description}
							</span>
						</div>
					</div>

					<a href={typeData?.homepage} target={'_blank'} rel={'noreferrer'}>
						<Button variant={'outline'}>
							<ExternalLinkIcon size={16} className={'mr-2'} />
							Learn More
						</Button>
					</a>
				</AlertDescription>
			</Alert>

			{(typeData?.experimental || typeData?.deprecated) && (
				<Alert className={'mb-2'} variant={'destructive'}>
					<AlertDescription>
						Keep in mind, <span className={'font-semibold'}>{typeData?.name}</span> is {typeData?.experimental ? 'experimental' : 'deprecated'} and may not work as expected. Take backups!
					</AlertDescription>
				</Alert>
			)}

			<div className={'flex flex-row items-center mb-2 w-full'}>
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

				<Select value={effectiveDisplayMode} onValueChange={(value) => setDisplayMode(value)} disabled={mobile}>
					<SelectTrigger className={'w-[11em] ml-2'}>
						<SelectValue placeholder={'Compact'} />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={'grid'}>Grid</SelectItem>
						<SelectItem value={'list'}>List</SelectItem>
						<SelectItem value={'compact'}>Compact</SelectItem>
					</SelectContent>
				</Select>

				<Input className={'ml-2'} placeholder={'Search Version'} value={search ?? ''} onChange={(e) => {
					setVersionPage(1)
					setSearch(e.target.value)
				}} />
			</div>

			<div className={'mb-3'}>
				{versionPaginationControls}
			</div>

			<div
				ref={versionsContainerRef}
				className={
					effectiveDisplayMode === 'grid'
						? 'grid md:grid-cols-2 grid-cols-1'
						: effectiveDisplayMode === 'list'
							? 'flex flex-col'
							: mobile
								? 'flex flex-col'
								: 'grid grid-cols-[repeat(auto-fit,minmax(30rem,1fr))]'
				}
			>
				{!filteredVersions ? (
					<>
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
						<Skeleton className={'h-16 rounded-xl mb-2 mx-1 xl:min-w-[30rem]'} />
					</>
				) : (
					<>
						{filteredVersions.map((version) => (
							<Card key={version.latest.versionId ?? version.latest.projectVersionId} className={'mb-2 mx-1 p-3 pr-4 xl:min-w-[30rem]'}>
								<div className={'flex flex-row items-center justify-between'}>
									<div className={'flex flex-row items-center'}>
										<img src={`https://s3.mcjars.app/icons/${type.toLowerCase()}.png`} alt={'Logo'} className={'h-12 w-12 rounded-md'} />
										<div className={'flex flex-col ml-2'}>
											<h1 className={'text-xl font-semibold flex flex-row items-center'}>
												{version.latest.versionId ?? version.latest.projectVersionId}
												<Badge className={'ml-2 md:block hidden'} variant={version.type === 'RELEASE' ? 'outline' : 'destructive'}>
													{version.type}
												</Badge>
												{version.type === 'SNAPSHOT' && (
													<ResponsiveTooltip>
														<Tooltip>
															<TooltipTrigger>
																<TriangleAlertIcon size={16} className={'ml-2 text-red-500 md:hidden'} />
															</TooltipTrigger>
															<TooltipContent>
																<p>SNAPSHOT</p>
															</TooltipContent>
														</Tooltip>
													</ResponsiveTooltip>
												)}
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

			<div className={'pb-2'}>
				{versionPaginationControls}
			</div>

			<Drawer open={Boolean(browse)} onClose={() => setBrowse(undefined)} setBackgroundColorOnScale={false}>
				<DrawerContent className={'w-full max-w-5xl mx-auto'} onPointerDownOutside={() => setBrowse(undefined)}>
					<DrawerHeader>
						<DrawerTitle>Browse {browse}</DrawerTitle>
					</DrawerHeader>
					<div className={'p-4 h-full max-h-96 overflow-y-auto'}>
						<div className={'mb-3 flex flex-row items-center'}>
							<Input placeholder={'Search Build'} value={buildSearch ?? ''} onChange={(e) => {
								setBuildPage(1)
								setBuildSearch(e.target.value)
							}} />
						</div>

						<div className={'mb-3'}>
							{buildPaginationControls}
						</div>
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
									<Card key={build.uuid} className={'mb-2 p-3 pr-4'}>
										<Collapsible defaultOpen={i === 0} className={'group/collapsible-build'}>
											<div className={'flex flex-row items-center justify-between'}>
												<div className={'flex flex-row items-center'}>
													<img src={`https://s3.mcjars.app/icons/${type.toLowerCase()}.png`} alt={'Logo'} className={'h-12 w-12 rounded-md'} />
													<div className={'flex flex-col ml-2'}>
														<h1 className={'text-xl font-semibold flex md:flex-row flex-col md:items-center items-start'}>
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

													<Button
														variant={'outline'}
														className={'mr-2'}
														onClick={() => {
															const params = new URLSearchParams()
															const selectedVersion = browse ?? build.versionId ?? build.projectVersionId
															if (selectedVersion) params.set('version', selectedVersion)
															if (build.uuid) params.set('build', build.uuid)
															navigate(`/${type}/config${params.toString() ? `?${params.toString()}` : ''}`)
														}}
													>
														Browse Configs
													</Button>

													<CollapsibleTrigger>
														<Button>
															Install
															<ChevronDown size={16} className={'ml-2 transition-transform group-data-[state=open]/collapsible-build:rotate-180'} />
														</Button>
													</CollapsibleTrigger>
												</div>
											</div>

											<CollapsibleContent className={'mt-2 flex flex-row items-end justify-between'}>
												<div className={'flex flex-col'}>
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
												</div>

												<div className={'w-3/5 hidden md:flex flex-row items-center'}>
													<Input
														value={installScript === 'bash'
															? `bash <(curl -s ${window.location.protocol}//${window.location.hostname}/install.sh) ${build.uuid}`
															: `mcvcli install --file=install --build=${build.uuid}`
														} disabled
													/>
													<Button variant={'outline'} className={'ml-2'} onClick={() => setInstallScript((prev) => prev === 'bash' ? 'mcvcli' : 'bash')}>
														<RefreshCwIcon size={16} />
													</Button>
												</div>
											</CollapsibleContent>
										</Collapsible>
									</Card>
								))}
								{buildPaginationControls}
							</>
						)}
					</div>
				</DrawerContent>
			</Drawer>
		</>
	)
}