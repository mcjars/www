import apiGetBuildConfigs, { BuildConfigItem } from "@/api/builds/configs"
import apiPostConfigFormat from "@/api/configs/format"
import { apiGetTypeVersionBuilds } from "@/api/builds/type-versions"
import apiGetTypes from "@/api/types"
import apiGetVersions from "@/api/versions"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Select, SelectContent, SelectItem, SelectTrigger } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { FileText, FolderOpen, LoaderCircle } from "lucide-react"
import { DragEvent as ReactDragEvent, PointerEvent as ReactPointerEvent, useEffect, useMemo, useRef, useState } from "react"
import { useParams } from "react-router-dom"
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer"
import useSWR from "swr"
import { StringParam, useQueryParam } from "use-query-params"
import { useLocalStorage } from "usehooks-ts"

type TreeNode = {
	folders: Record<string, TreeNode>
	files: BuildConfigItem[]
}

const normalizeLocation = (value?: string) => (value ?? "").trim().replace(/\/+/g, "/")
const normalizeTypeId = (value?: string) => (value ?? "").trim().toUpperCase()
const isUuid = (value?: string | null) => Boolean(value && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value))

const buildTree = (items: BuildConfigItem[]) => {
	const root: TreeNode = { folders: {}, files: [] }

	for (const item of items) {
		const location = normalizeLocation(item.location)
		if (!location) continue

		const parts = location.split("/").filter(Boolean)
		let node = root

		for (let index = 0; index < parts.length; index++) {
			const part = parts[index]
			const isLeaf = index === parts.length - 1

			if (isLeaf) {
				node.files.push(item)
			} else {
				node.folders[part] = node.folders[part] ?? { folders: {}, files: [] }
				node = node.folders[part]
			}
		}
	}

	const sortNode = (node: TreeNode) => {
		node.files.sort((a, b) => normalizeLocation(a.location).localeCompare(normalizeLocation(b.location)))
		for (const child of Object.values(node.folders)) {
			sortNode(child)
		}
	}

	sortNode(root)
	return root
}

const countFiles = (node: TreeNode): number => {
	let count = node.files.length
	for (const child of Object.values(node.folders)) {
		count += countFiles(child)
	}
	return count
}

const isSameConfig = (left: BuildConfigItem | null, right: BuildConfigItem) => {
	if (!left) return false
	if (left.valueUuid && right.valueUuid) return left.valueUuid === right.valueUuid
	if (left.configUuid && right.configUuid) return left.configUuid === right.configUuid
	return normalizeLocation(left.location) === normalizeLocation(right.location)
}

const inferFormatKey = (selected: BuildConfigItem | null) => {
	const format = (selected?.format ?? "").toUpperCase()
	if (["JSON", "JSON5"].includes(format)) return "json"
	if (["YAML", "YML"].includes(format)) return "yaml"
	if (["TOML"].includes(format)) return "toml"
	if (["PROPERTIES", "CONF", "CFG"].includes(format)) return "properties"

	const location = normalizeLocation(selected?.location).toLowerCase()
	if (location.endsWith(".json") || location.endsWith(".json5")) return "json"
	if (location.endsWith(".yaml") || location.endsWith(".yml")) return "yaml"
	if (location.endsWith(".toml")) return "toml"
	if (location.endsWith(".properties") || location.endsWith(".conf") || location.endsWith(".cfg")) return "properties"

	return "plain"
}

const renderHighlightedLine = (line: string, formatKey: string) => {
	if (formatKey === "json") {
		const keyValue = line.match(/^(\s*"[^"]+"\s*:\s*)(.*)$/)
		if (keyValue) {
			const rawValue = keyValue[2]
			const value = rawValue.trim()
			const valueClass =
				value.startsWith('"') ? "text-emerald-300" :
					/^[-]?\d/.test(value) ? "text-orange-300" :
						/^(true|false|null)/.test(value) ? "text-cyan-300" :
							"text-sidebar-foreground"

			return (
				<>
					<span className={"text-purple-300"}>{keyValue[1]}</span>
					<span className={valueClass}>{rawValue || " "}</span>
				</>
			)
		}

		if (/^\s*\/\//.test(line)) {
			return <span className={"text-sidebar-foreground/60"}>{line || " "}</span>
		}

		return <span className={"text-sidebar-foreground"}>{line || " "}</span>
	}

	if (formatKey === "yaml" || formatKey === "toml" || formatKey === "properties") {
		if (/^\s*[#;]/.test(line)) {
			return <span className={"text-sidebar-foreground/60"}>{line || " "}</span>
		}

		const keyValue = line.match(/^(\s*[^:=\s][^:=]*\s*[:=])(\s*)(.*)$/)
		if (keyValue) {
			const value = keyValue[3].trim()
			const valueClass =
				value.startsWith('"') || value.startsWith("'") ? "text-emerald-300" :
					/^[-]?\d/.test(value) ? "text-orange-300" :
						/^(true|false|null)/i.test(value) ? "text-cyan-300" :
							"text-sidebar-foreground"

			return (
				<>
					<span className={"text-purple-300"}>{keyValue[1]}</span>
					<span className={"text-sidebar-foreground/80"}>{keyValue[2]}</span>
					<span className={valueClass}>{keyValue[3] || " "}</span>
				</>
			)
		}

		return <span className={"text-sidebar-foreground"}>{line || " "}</span>
	}

	return <span className={"text-sidebar-foreground"}>{line || " "}</span>
}

const getAllVersionIdsForType = async (type: string): Promise<string[]> => {
	const perPage = 200
	const maxPages = 20
	const collected: string[] = []
	const seen = new Set<string>()

	for (let page = 1; page <= maxPages; page++) {
		const response = await apiGetVersions(type, { page, perPage, search: "" })
		for (const item of response.items ?? []) {
			const versionId = item?.latest?.versionId ?? item?.latest?.projectVersionId
			if (versionId && !seen.has(versionId)) {
				seen.add(versionId)
				collected.push(versionId)
			}
		}

		if (!response.hasNextPage) break
	}

	return collected
}

const renderTreeWithSelection = (
	node: TreeNode,
	depth: number,
	path: string,
	selected: BuildConfigItem | null,
	onSelect: (item: BuildConfigItem) => void
) => {
	const folderEntries = Object.entries(node.folders).sort(([a], [b]) => a.localeCompare(b))

	return (
		<div className={"w-full text-sidebar-foreground"}>
			{folderEntries.length > 0 && (
				<Accordion type={"multiple"} className={"w-full"} defaultValue={folderEntries.map(([name]) => `${path}/${name}`)}>
					{folderEntries.map(([folderName, childNode]) => {
						const folderPath = path ? `${path}/${folderName}` : folderName
						const fileCount = countFiles(childNode)

						return (
							<AccordionItem key={folderPath} value={folderPath} className={"border-none"}>
								<AccordionTrigger className={"px-0 py-0 hover:no-underline"}>
									<div className={"flex w-full items-center justify-between gap-3 rounded-lg px-2 py-2 hover:bg-sidebar-accent/60"} style={{ paddingLeft: depth * 16 }}>
										<div className={"flex min-w-0 items-center gap-3"}>
											<div className={"flex h-8 w-8 ml-2 items-center justify-center rounded-md border border-sidebar-border/80 bg-sidebar/60 text-sidebar-foreground/80"}>
												<FolderOpen size={14} className={"shrink-0"} />
											</div>
											<div className={"min-w-0"}>
												<p className={"truncate font-medium"}>{folderName}</p>
												<p className={"text-xs text-sidebar-foreground/70"}>{fileCount} file{fileCount === 1 ? "" : "s"}</p>
											</div>
										</div>

										<div className={"mr-2 flex shrink-0 items-center gap-2"}>
											<Badge variant={"secondary"} className={"bg-sidebar-accent/70 text-sidebar-accent-foreground"}>{fileCount}</Badge>
										</div>
									</div>
								</AccordionTrigger>

								<AccordionContent className={"pt-1 pb-2"}>
									<div className={"space-y-1"}>
										{renderTreeWithSelection(childNode, depth + 1, folderPath, selected, onSelect)}
									</div>
								</AccordionContent>
							</AccordionItem>
						)
					})}
				</Accordion>
			)}

			<div className={"space-y-1"}>
				{node.files.map((item) => {
					const location = normalizeLocation(item.location)
					const parts = location.split("/").filter(Boolean)
					const fileName = parts.length > 0 ? parts[parts.length - 1] : location
					const selectedItem = isSameConfig(selected, item)

					return (
						<div
							key={item.valueUuid || item.configUuid || location}
							role={"button"}
							tabIndex={0}
							onClick={() => onSelect(item)}
							onKeyDown={(event) => {
								if (event.key === "Enter" || event.key === " ") {
									event.preventDefault()
									onSelect(item)
								}
							}}
							className={cn(
								"group w-full cursor-pointer touch-pan-y rounded-lg px-2 py-2 transition-colors",
								selectedItem
									? "bg-sidebar-accent text-sidebar-accent-foreground"
									: "hover:bg-sidebar-accent/60"
							)}
							style={{ paddingLeft: depth * 16 }}
						>
							<div className={"flex min-w-0 items-center gap-3"}>
								<div className={cn(
									"ml-2 flex h-8 w-8 items-center justify-center rounded-md border text-sidebar-foreground",
									selectedItem
										? "border-sidebar-border bg-sidebar-accent-foreground/10"
										: "border-sidebar-border/80 bg-sidebar/60 group-hover:bg-sidebar"
								)}>
									<FileText size={14} className={"shrink-0"} />
								</div>
								<p className={"truncate font-medium"}>{fileName || item.location}</p>
							</div>
						</div>
					)
				})}
			</div>
		</div>
	)
}

export default function PageTypeConfig() {
	const { type } = useParams<{ type: string }>()
	if (!type) return null
	const normalizedType = normalizeTypeId(type)

	const [browse] = useQueryParam("browse", StringParam)
	const [versionParam, setVersionParam] = useQueryParam("version", StringParam)
	const [buildParam, setBuildParam] = useQueryParam("build", StringParam)
	const [selected, setSelected] = useState<BuildConfigItem | null>(null)
	const [isMobile, setIsMobile] = useState(false)
	const [mobilePane, setMobilePane] = useState<"files" | "editor">("files")
	const [filesPaneWidth, setFilesPaneWidth] = useLocalStorage<number>("type-config-files-pane-width", 360)
	const [localCompare, setLocalCompare] = useState<{ name: string, formatted: string } | null>(null)
	const [isFormattingLocal, setIsFormattingLocal] = useState(false)
	const [compareError, setCompareError] = useState<string | null>(null)
	const [isCompareDragOver, setIsCompareDragOver] = useState(false)
	const layoutRef = useRef<HTMLDivElement | null>(null)
	const compareInputRef = useRef<HTMLInputElement | null>(null)
	const compareDragDepthRef = useRef(0)

	useEffect(() => {
		const media = window.matchMedia("(max-width: 1023px)")
		const apply = () => setIsMobile(media.matches)
		apply()
		media.addEventListener("change", apply)
		return () => media.removeEventListener("change", apply)
	}, [])

	const { data: types } = useSWR(
		["types"],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: versionOptions } = useSWR(
		["versions-all", normalizedType],
		() => getAllVersionIdsForType(normalizedType),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)
	const resolvedVersionOptions = versionOptions ?? []

	const selectedVersion = useMemo(() => {
		if (versionParam && resolvedVersionOptions.includes(versionParam)) return versionParam
		if (browse && !isUuid(browse) && resolvedVersionOptions.includes(browse)) return browse
		return resolvedVersionOptions[0] ?? null
	}, [versionParam, browse, resolvedVersionOptions])

	useEffect(() => {
		if (!versionParam && selectedVersion) {
			setVersionParam(selectedVersion)
		}
	}, [selectedVersion, versionParam, setVersionParam])

	const { data: buildItems } = useSWR(
		selectedVersion ? ["type-version-builds", normalizedType, selectedVersion] : null,
		() => apiGetTypeVersionBuilds(normalizedType, selectedVersion as string),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const buildOptions = useMemo(() => {
		const seen = new Set<string>()
		return (buildItems ?? []).filter((item) => {
			if (!item.uuid || seen.has(item.uuid)) return false
			seen.add(item.uuid)
			return true
		})
	}, [buildItems])

	const selectedBuildUuid = useMemo(() => {
		if (buildParam && buildOptions.some((item) => item.uuid === buildParam)) return buildParam
		if (browse && isUuid(browse) && buildOptions.some((item) => item.uuid === browse)) return browse
		return buildOptions[0]?.uuid ?? null
	}, [buildParam, browse, buildOptions])

	useEffect(() => {
		if (!buildParam && selectedBuildUuid) {
			setBuildParam(selectedBuildUuid)
		}
	}, [selectedBuildUuid, buildParam, setBuildParam])

	const { data: configs } = useSWR(
		selectedBuildUuid ? ["build-configs", selectedBuildUuid] : null,
		() => apiGetBuildConfigs(selectedBuildUuid as string),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	useEffect(() => {
		if (!selected && configs && configs.length > 0) {
			setSelected(configs[0])
		}
	}, [selected, configs])

	const handleSelectFile = (item: BuildConfigItem) => {
		setSelected(item)
		if (isMobile) {
			setMobilePane("editor")
		}
	}

	const typeData = useMemo(
		() => Object.values(types ?? {}).flat().find((entry: any) => normalizeTypeId(entry.identifier) === normalizeTypeId(type)),
		[types, type]
	)

	const selectedBuild = useMemo(
		() => buildOptions.find((item) => item.uuid === selectedBuildUuid) ?? null,
		[buildOptions, selectedBuildUuid]
	)
	const selectedConfigKey = useMemo(
		() => selected?.valueUuid ?? selected?.configUuid ?? normalizeLocation(selected?.location),
		[selected]
	)

	const selectedBuildLabel = useMemo(() => {
		if (!selectedBuild) return "Select build"
		const versionLabel = selectedBuild.versionId ?? selectedBuild.projectVersionId ?? "Unknown"
		const nameLabel = selectedBuild.name?.trim()
		return nameLabel ? `${versionLabel} - ${nameLabel}` : versionLabel
	}, [selectedBuild])

	const tree = useMemo(() => buildTree(configs ?? []), [configs])
	const fileCount = useMemo(() => countFiles(tree), [tree])
	const selectedLines = useMemo(() => (selected?.value ?? "").split("\n"), [selected])
	const formatKey = useMemo(() => inferFormatKey(selected), [selected])

	const [viewerWidth, setViewerWidth] = useState(1600)
	const viewerRef = useRef<HTMLDivElement | null>(null)

	useEffect(() => {
		const updateWidth = () => {
			setViewerWidth(viewerRef.current?.clientWidth ?? window.innerWidth)
		}

		updateWidth()
		window.addEventListener("resize", updateWidth)

		const observer = new ResizeObserver(updateWidth)
		if (viewerRef.current) observer.observe(viewerRef.current)

		return () => {
			window.removeEventListener("resize", updateWidth)
			observer.disconnect()
		}
	}, [])

	const isNarrowDiff = viewerWidth < 1400

	useEffect(() => {
		if (!selected || !configs?.length) return
		const stillExists = configs.some((item) => isSameConfig(selected, item))
		if (!stillExists) {
			setSelected(configs[0])
		}
	}, [selected, configs])

	useEffect(() => {
		const clamped = Math.min(720, Math.max(200, filesPaneWidth))
		if (clamped !== filesPaneWidth) {
			setFilesPaneWidth(clamped)
		}
	}, [filesPaneWidth, setFilesPaneWidth])

	const startResize = (event: ReactPointerEvent<HTMLDivElement>) => {
		if (isMobile) return

		const container = layoutRef.current
		if (!container) return

		const rect = container.getBoundingClientRect()
		const minWidth = 200
		const maxWidth = Math.max(minWidth, Math.min(720, rect.width - 340))

		const onMove = (moveEvent: PointerEvent) => {
			const next = Math.min(maxWidth, Math.max(minWidth, moveEvent.clientX - rect.left))
			setFilesPaneWidth(Math.round(next))
		}

		const onUp = () => {
			window.removeEventListener("pointermove", onMove)
			window.removeEventListener("pointerup", onUp)
		}

		window.addEventListener("pointermove", onMove)
		window.addEventListener("pointerup", onUp)
		event.preventDefault()
	}

	const clearComparison = () => {
		setLocalCompare(null)
		setCompareError(null)
		setIsCompareDragOver(false)
		compareDragDepthRef.current = 0
	}

	useEffect(() => {
		clearComparison()
	}, [normalizedType, selectedVersion, selectedBuildUuid, selectedConfigKey])

	const handleCompareFile = async (file: File) => {
		setCompareError(null)
		setIsFormattingLocal(true)
		const raw = await file.text()

		setLocalCompare({
			name: file.name,
			formatted: raw
		})

		try {
			const formatted = await Promise.race([
				apiPostConfigFormat(file, raw),
				new Promise<string>((_, reject) => setTimeout(() => reject(new Error("format-timeout")), 2500))
			])

			if (formatted) {
				setLocalCompare({
					name: file.name,
					formatted
				})
			} else {
				setCompareError("Formatter returned empty content; using raw file.")
			}
		} catch {
			setCompareError("Formatter unavailable; using raw file for comparison.")
		} finally {
			setIsFormattingLocal(false)
		}
	}

	const handleCompareDragEnter = (event: ReactDragEvent<HTMLDivElement>) => {
		event.preventDefault()
		event.stopPropagation()
		compareDragDepthRef.current += 1
		setIsCompareDragOver(true)
	}

	const handleCompareDragLeave = (event: ReactDragEvent<HTMLDivElement>) => {
		event.preventDefault()
		event.stopPropagation()
		compareDragDepthRef.current = Math.max(0, compareDragDepthRef.current - 1)
		if (compareDragDepthRef.current === 0) {
			setIsCompareDragOver(false)
		}
	}

	const handleCompareDrop = (event: ReactDragEvent<HTMLDivElement>) => {
		event.preventDefault()
		event.stopPropagation()
		compareDragDepthRef.current = 0
		setIsCompareDragOver(false)

		const file = event.dataTransfer.files?.[0]
		if (!file) return
		handleCompareFile(file)
	}

	return (
		<div className={"flex h-full min-h-0 flex-col overflow-hidden"}>
			<input
				type={"file"}
				accept={".yml,.yaml,.properties,.toml,.conf,.cfg,.json,.json5"}
				className={"hidden"}
				ref={compareInputRef}
				onChange={(event) => {
					const file = event.target.files?.[0]
					if (!file) return
					handleCompareFile(file)
					event.target.value = ""
				}}
			/>

			<div className={"mb-2 flex shrink-0 items-center justify-between gap-4"}>
				<div className={"min-w-0"}>
					<h1 className={"truncate text-2xl font-semibold"}>{typeData?.name ?? type}</h1>
					<p className={"text-sm text-muted-foreground"}>
						{selectedBuildUuid ? `${fileCount} file${fileCount !== 1 ? "s" : ""} for selected build` : "Select a version and build to view configs"}
					</p>
				</div>

				<div className={"grid w-full max-w-xl grid-cols-1 gap-2 sm:grid-cols-2"}>
					<Select value={selectedVersion ?? undefined} onValueChange={(value) => {
						setVersionParam(value)
						setBuildParam(undefined)
					}}>
						<SelectTrigger className={"w-full"}>
							<div className={"flex min-w-0 items-center gap-2 text-left"}>
								<span className={"text-xs text-muted-foreground"}>Version</span>
								<span className={"truncate"}>{selectedVersion ?? "Select version"}</span>
							</div>
						</SelectTrigger>
						<SelectContent>
							{resolvedVersionOptions.map((versionId) => (
								<SelectItem key={versionId} value={versionId}>{versionId}</SelectItem>
							))}
						</SelectContent>
					</Select>

					<Select value={selectedBuildUuid ?? undefined} onValueChange={setBuildParam}>
						<SelectTrigger className={"w-full"}>
							<div className={"flex min-w-0 items-center gap-2 text-left"}>
								{typeData?.icon ? <img src={typeData.icon} alt={typeData.name} className={"h-5 w-5 rounded-md"} /> : null}
								<span className={"truncate"}>{selectedBuildLabel}</span>
							</div>
						</SelectTrigger>
						<SelectContent>
							{buildOptions.map((build) => (
								<SelectItem key={build.uuid} value={build.uuid}>
									{build.versionId ?? build.projectVersionId ?? "Unknown"} - {build.name || "Unnamed Build"}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
			</div>

			{!types || !versionOptions || !buildItems || !configs ? (
				<div className={"w-full"}>
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
				</div>
			) : !selectedBuildUuid ? (
				<Card className={"p-6 text-sm text-muted-foreground"}>No build UUID available for this version.</Card>
			) : (
				<div className={"flex min-h-0 flex-1 flex-col gap-2 overflow-hidden"}>
					<div className={"grid grid-cols-2 gap-2 lg:hidden"}>
						<Button variant={mobilePane === "files" ? "default" : "outline"} onClick={() => setMobilePane("files")}>Files</Button>
						<Button variant={mobilePane === "editor" ? "default" : "outline"} onClick={() => setMobilePane("editor")}>Editor</Button>
					</div>

					<div ref={layoutRef} className={"flex min-h-0 flex-1 flex-col gap-2 overflow-hidden lg:flex-row lg:gap-0"}>
						<div
							className={cn("relative min-h-0 lg:shrink-0", isMobile && mobilePane !== "files" ? "hidden" : "flex")}
							style={!isMobile ? { width: `${filesPaneWidth}px` } : undefined}
						>
							<Card className={"min-h-0 w-full overflow-hidden rounded-xl border-sidebar-border bg-sidebar p-0 text-sidebar-foreground flex flex-col lg:rounded-l-xl lg:rounded-r-none lg:border-r-0"}>
								<div className={"border-b border-sidebar-border px-4 py-4 font-medium text-sidebar-foreground"}>
									Files
								</div>
								<div className={"min-h-0 flex-1 overflow-hidden p-2"}>
									{renderTreeWithSelection(tree, 0, normalizedType, selected, handleSelectFile)}
								</div>
							</Card>

							{!isMobile && (
								<div
									role={"separator"}
									aria-orientation={"vertical"}
									onPointerDown={startResize}
									className={"absolute right-0 top-0 hidden h-full w-2 translate-x-1/2 cursor-col-resize rounded-full bg-transparent transition-colors lg:block"}
								/>
							)}
						</div>

						<Card
							className={cn("relative min-h-0 flex-1 overflow-hidden rounded-xl bg-sidebar p-0 lg:rounded-l-none lg:rounded-r-xl border-sidebar-border", isMobile && mobilePane !== "editor" ? "hidden" : "flex flex-col")}
							onDragEnter={handleCompareDragEnter}
							onDragLeave={handleCompareDragLeave}
							onDragOver={(event) => {
								event.preventDefault()
								event.stopPropagation()
								event.dataTransfer.dropEffect = "copy"
							}}
							onDrop={handleCompareDrop}
						>
							{isCompareDragOver && (
								<div className={"pointer-events-none absolute inset-0 z-20 flex items-center justify-center border-2 border-dashed border-primary/70 bg-sidebar-accent/30 text-sm font-medium text-sidebar-accent-foreground"}>
									Drop local config to compare
								</div>
							)}
							<div className={cn("border-b border-sidebar-border px-4 font-medium text-sidebar-foreground", localCompare ? "py-2" : "py-3")}>
								{selected ? (
									<div className={"flex min-w-0 items-center justify-between gap-3"}>
										<div className={"min-w-0"}>
											<p className={"truncate font-medium"}>{selected.location}</p>
											{localCompare && <p className={"truncate text-xs text-muted-foreground"}>Comparing with {localCompare.name}</p>}
											{compareError && <p className={"text-xs text-amber-400"}>{compareError}</p>}
										</div>
										<div className={"shrink-0 flex items-center gap-2"}>
											<Button
												variant={"outline"}
												size={"sm"}
												onClick={() => compareInputRef.current?.click()}
												disabled={isFormattingLocal}
											>
												{isFormattingLocal ? <LoaderCircle size={14} className={"mr-2 animate-spin"} /> : null}
												Compare Local
											</Button>
											{localCompare && (
												<Button variant={"ghost"} size={"sm"} onClick={() => {
													clearComparison()
												}}>
													Clear Comparison
												</Button>
											)}
											<Badge variant={"outline"}>{selected.format}</Badge>
											<Badge variant={"secondary"}>{selected.value ? selected.value.split("\n").length : 0} lines</Badge>
										</div>
									</div>
								) : (
									<p className={"text-sm text-muted-foreground"}>Select a file from the tree to preview its contents.</p>
								)}
							</div>

							<div ref={viewerRef} className={"min-h-0 flex-1 overflow-auto bg-sidebar-background p-2"}>
								{selected ? (
									localCompare ? (
										<ReactDiffViewer
											splitView={!isNarrowDiff}
											useDarkTheme
											showDiffOnly={isNarrowDiff}
											extraLinesSurroundingDiff={isNarrowDiff ? 1 : 3}
											hideLineNumbers={isNarrowDiff}
											oldValue={localCompare.formatted}
											newValue={selected.value}
											compareMethod={DiffMethod.LINES}
											leftTitle={"Local File"}
											rightTitle={"MCJars Config"}
											styles={{
												diffContainer: { background: "hsl(var(--sidebar-background))" },
												contentText: {
													color: "hsl(var(--sidebar-foreground))",
													fontSize: isNarrowDiff ? "12px" : "13px",
													wordBreak: "break-word",
													whiteSpace: "pre-wrap"
												},
												line: { wordBreak: "break-word" },
												titleBlock: {
													background: "hsl(var(--sidebar-background))",
													color: "hsl(var(--sidebar-foreground))"
												},
												lineNumber: {
													color: "hsl(var(--sidebar-foreground))",
													opacity: 0.9
												},
												gutter: {
													background: "hsl(var(--sidebar-background))",
													color: "hsl(var(--sidebar-foreground))"
												},
												emptyLine: {
													background: "hsl(var(--sidebar-background))"
												}
											}}
										/>
									) : (
										<div className={"min-w-full font-mono text-xs leading-5"}>
											{selectedLines.map((line, index) => (
												<div key={`${selected.valueUuid}-${index}`} className={"flex"}>
													<span className="select-none bg-sidebar px-3 text-right text-muted-foreground w-12 shrink-0">{index + 1}</span>
													<code className="block w-full whitespace-pre-wrap px-3">{renderHighlightedLine(line, formatKey)}</code>
												</div>
											))}
										</div>
									)
								) : (
									<div className={"text-sm text-muted-foreground"}>No file selected.</div>
								)}
							</div>
						</Card>
					</div>
				</div>
			)}
		</div>
	)
}
