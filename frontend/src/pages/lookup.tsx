import apiGetBuild from "@/api/builds/by-hash"
import { PartialMinecraftBuild } from "@/api/builds"
import apiGetConfigSearch from "@/api/configs/identify"
import apiGetTypes from "@/api/types"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Drawer, DrawerContent } from "@/components/ui/drawer"
import { Select, SelectContent, SelectItem, SelectTrigger } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import bytes from "bytes"
import { LoaderCircle } from "lucide-react"
import { LegacyRef, useEffect, useMemo, useRef, useState } from "react"
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer"
import useSWR from "swr"

export default function PageLookup() {
	const [isDragging, setIsDragging] = useState(false)
	const [isDropLoading, setIsDropLoading] = useState(false)
	const [jarDropBuild, setJarDropBuild] = useState<PartialMinecraftBuild>()
	const [configDropMatches, setConfigDropMatches] = useState<Awaited<ReturnType<typeof apiGetConfigSearch>>>()
	const [configDropMatchIndex, setConfigDropMatchIndex] = useState(0)
	const [viewerWidth, setViewerWidth] = useState(1920)
	const inputRef = useRef<HTMLInputElement>()
	const viewerRef = useRef<HTMLDivElement>(null)

	const { data: types } = useSWR(
		['types'],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const handleFile = async (file: File) => {
		if (file.name.endsWith('.jar')) {
			setIsDropLoading(true)

			const hash = await crypto.subtle.digest('SHA-256', await file.arrayBuffer()),
				hashArray = Array.from(new Uint8Array(hash)),
				hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')

			try {
				const build = await apiGetBuild(hashHex)

				setJarDropBuild(build)
				setIsDropLoading(false)
			} catch {
				setIsDropLoading(false)
				setJarDropBuild({
					uuid: '',
					type: 'UNKNOWN',
					name: '???',
					changes: [],
					created: null,
					experimental: false,
					installation: [],
					projectVersionId: null,
					versionId: 'Unknown'
				})
			}
		} else if (
			file.name.endsWith('.yml') ||
			file.name.endsWith('.properties') ||
			file.name.endsWith('.toml') ||
			file.name.endsWith('.conf') ||
			file.name.endsWith('.json') ||
			file.name.endsWith('.json5')
		) {
			setIsDropLoading(true)

			const config = await apiGetConfigSearch(file)

			setIsDropLoading(false)
			setConfigDropMatches(config)
		}
	}

	useEffect(() => {
		if (configDropMatches) {
			setConfigDropMatchIndex(0)
		}
	}, [configDropMatches])

	useEffect(() => {
		const updateWidth = () => {
			setViewerWidth(viewerRef.current?.clientWidth ?? window.innerWidth)
		}

		updateWidth()
		window.addEventListener('resize', updateWidth)

		const observer = new ResizeObserver(updateWidth)
		if (viewerRef.current) {
			observer.observe(viewerRef.current)
		}

		return () => {
			window.removeEventListener('resize', updateWidth)
			observer.disconnect()
		}
	}, [])

	useEffect(() => {
		const handleDragEnter = (e: DragEvent) => {
			e.preventDefault()
			setIsDragging(true)
		}

		const handleDragOver = (e: DragEvent) => {
			e.preventDefault()
			setIsDragging(true)
		}

		const handleDragLeave = (e: DragEvent) => {
			e.preventDefault()
			setIsDragging(false)
		}

		const handleDrop = async (e: DragEvent) => {
			e.preventDefault()
			setIsDragging(false)

			const file = e.dataTransfer?.files[0]
			if (!file) return

			await handleFile(file)
		}

		window.addEventListener('dragenter', handleDragEnter)
		window.addEventListener('dragover', handleDragOver)
		window.addEventListener('dragleave', handleDragLeave)
		window.addEventListener('drop', handleDrop)

		return () => {
			window.removeEventListener('dragenter', handleDragEnter)
			window.removeEventListener('dragover', handleDragOver)
			window.removeEventListener('dragleave', handleDragLeave)
			window.removeEventListener('drop', handleDrop)
		}
	}, [])

	const typeData = useMemo(
		() => Object.values(types ?? {}).flat()?.find((t) => t.identifier === jarDropBuild?.type),
		[types, jarDropBuild]
	)

	const configDropMatch = configDropMatches?.configs[configDropMatchIndex]
	const configDropMatchTypeData = useMemo(
		() => Object.values(types ?? {}).flat().find((t) => t.identifier === configDropMatch?.from),
		[types, configDropMatch]
	)
	const isNarrowDiff = viewerWidth < 1400

	return (
		<>
			<input
				id={'file-input'}
				type={'file'}
				className={'hidden'}
				ref={inputRef as LegacyRef<HTMLInputElement>}
				accept={'.jar,.yml,.properties,.toml,.conf,.json,.json5'}
				onChange={(e) => {
					const file = e.target.files?.[0]
					if (!file) return

					handleFile(file)
				}}
			/>

			{!configDropMatches ? (
				<div className={'flex flex-col items-center justify-center h-full w-full text-center'}>
					{isDropLoading ? (
						<LoaderCircle className={'animate-spin'} />
					) : (
						<>
							<h1 className={'text-4xl font-semibold'}>Drag in your file to look up!</h1>
							<p className={'text-lg cursor-pointer hover:underline'} onClick={() => inputRef.current?.click()}>
								Or click this text.
							</p>
						</>
					)}
				</div>
			) : (
				<div ref={viewerRef} className={'w-full h-full pb-4 flex flex-col overflow-hidden'}>
					<div className={'mb-2 flex flex-col md:flex-row md:items-center md:justify-between sticky top-0 z-20 bg-background py-1 gap-2'}>
						<div>
							<h1 className={'text-xl md:text-2xl font-semibold'}>Config Diff Viewer</h1>
							<p className={'text-muted-foreground text-sm'}>Matched {configDropMatches.configs.length} best configs from your uploaded file.</p>
						</div>
						<div className={'flex flex-row gap-2 md:self-auto self-end'}>
							<Button variant={'outline'} onClick={() => inputRef.current?.click()}>
								Upload another file
							</Button>
							<Button variant={'destructive'} onClick={() => setConfigDropMatches(undefined)}>
								Close viewer
							</Button>
						</div>
					</div>

					<Card className={'p-2 mb-2'}>
						<div className={'flex flex-col min-[480px]:flex-row min-[480px]:items-center gap-2'}>
							<p className={'text-sm text-muted-foreground min-[480px]:min-w-[9rem]'}>Matched build</p>
							<Select value={String(configDropMatchIndex)} onValueChange={(value) => setConfigDropMatchIndex(Number(value))}>
								<SelectTrigger className={'w-full'}>
									<div className={'flex flex-row items-center min-w-0'}>
										{configDropMatchTypeData?.icon && (
											<img src={configDropMatchTypeData.icon} alt={configDropMatchTypeData.name} className={'h-5 w-5 rounded-md mr-2 shrink-0'} />
										)}
										<span className={'truncate text-left'}>
											{configDropMatchTypeData?.name ?? configDropMatch?.from ?? 'Select a matched build'}
											{configDropMatch?.build?.versionId ? ` - ${configDropMatch.build.versionId}` : ''}
										</span>
									</div>
								</SelectTrigger>
								<SelectContent>
									{configDropMatches.configs.map((config, index) => {
										const configTypeData = Object.values(types ?? {}).flat().find((t) => t.identifier === config.from)
										return (
											<SelectItem key={`${config.build?.uuid ?? config.from}-${index}`} value={String(index)}>
												{configTypeData?.name ?? config.from} {config.build?.versionId ? `- ${config.build.versionId}` : ''}
											</SelectItem>
										)
									})}
								</SelectContent>
							</Select>
						</div>
					</Card>

					<div className={'flex-1 min-h-0'}>
						<Card className={'overflow-auto min-h-0 h-full'}>
							{configDropMatch && (
								<ReactDiffViewer
									splitView={!isNarrowDiff}
									useDarkTheme
									showDiffOnly={isNarrowDiff}
									extraLinesSurroundingDiff={isNarrowDiff ? 1 : 3}
									hideLineNumbers={isNarrowDiff}
									oldValue={configDropMatches.formatted}
									newValue={configDropMatch.value}
									compareMethod={DiffMethod.LINES}
									leftTitle={'Uploaded Config'}
									rightTitle={`${configDropMatchTypeData?.name ?? configDropMatch.from} Match`}
									styles={{
										diffContainer: {
											background: 'hsl(var(--sidebar-background))'
										},
										contentText: {
											color: 'hsl(var(--sidebar-foreground))',
											fontSize: isNarrowDiff ? '12px' : '13px',
											wordBreak: 'break-word',
											whiteSpace: 'pre-wrap'
										},
										line: {
											wordBreak: 'break-word'
										},
										titleBlock: {
											background: 'hsl(var(--sidebar-background))',
											color: 'hsl(var(--sidebar-foreground))'
										},
										lineNumber: {
											color: 'hsl(var(--sidebar-foreground))',
											opacity: 0.9
										},
										gutter: {
											background: 'hsl(var(--sidebar-background))',
											color: 'hsl(var(--sidebar-foreground))',
											'&:hover': {
												background: 'hsl(var(--sidebar-accent))'
											}
										},
										emptyLine: {
											background: 'hsl(var(--sidebar-background))'
										}
									}}
								/>
							)}
						</Card>
					</div>
				</div>
			)}

			<Drawer open={isDragging || Boolean(jarDropBuild)} onOpenChange={(open) => {
				if (isDropLoading) return

				setIsDragging(open)

				if (!open) {
					setJarDropBuild(undefined)
				}
			}}>
				<DrawerContent className={'w-full max-w-3xl mx-auto'}>
					{jarDropBuild ? (
						<div className={'flex flex-row justify-between items-center p-2'}>
							<div className={'flex flex-row'}>
								<img src={typeData?.icon ?? 'https://s3.mcjars.app/icons/vanilla.png'} alt={jarDropBuild.type ?? undefined} className={'h-24 w-24 mr-2 rounded-md'} />
								<div className={'flex flex-col items-start'}>
									<h1 className={'text-xl font-semibold'}>{typeData?.name ?? 'Unknown'}</h1>
									<h1 className={'text-xl'}>{jarDropBuild.name}</h1>
									<p>{bytes(jarDropBuild.installation.flat().filter((i) => i.type === 'download').reduce((a, b) => a + b.size, 0))}</p>
								</div>
							</div>
							<div className={'flex flex-col items-end w-48 h-full mr-2'}>
								<p>{jarDropBuild.created}</p>
								{jarDropBuild.versionId && <h1 className={'text-xl'}>Minecraft {jarDropBuild.versionId}</h1>}
								{jarDropBuild.projectVersionId && <h1 className={'text-xl'}>{jarDropBuild.projectVersionId}</h1>}
							</div>
						</div>
					) : (
						<div className={'flex flex-row justify-between items-center p-2'}>
							<div className={'flex flex-row'}>
								<Skeleton className={'h-24 w-24 mr-2 rounded-md'} />
							</div>
							<div />
						</div>
					)}
				</DrawerContent>
			</Drawer>
		</>
	)
}