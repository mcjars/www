import apiGetConfigs, { ConfigItem } from "@/api/configs"
import apiGetTypes from "@/api/types"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
// Drawer component temporarily commented out, will be used later
// import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from "@/components/ui/drawer"
import { Skeleton } from "@/components/ui/skeleton"
import { FileText, FolderOpen } from "lucide-react"
import { useMemo, useState } from "react"
import useSWR from "swr"

type KnownType = {
	identifier: string
	name: string
	icon: string
	builds: number
}

type TreeNode = {
	folders: Record<string, TreeNode>
	files: ConfigItem[]
}

const normalizeTypeId = (value?: string) => (value ?? "").trim().toUpperCase()
const normalizeLocation = (value?: string) => (value ?? "").trim().replace(/\/+/g, "/")

const getConfigTypeIds = (config: ConfigItem): string[] => {
	const fromTypes = Array.isArray(config.types) ? config.types : []
	const fallback = typeof config.type === "string" && config.type ? [config.type] : []
	const raw = fromTypes.length > 0 ? fromTypes : fallback

	return Array.from(new Set(raw.map((value) => normalizeTypeId(value)).filter(Boolean)))
}

const buildTree = (list: ConfigItem[]) => {
	const root: TreeNode = { folders: {}, files: [] }

	for (const config of list) {
		const location = normalizeLocation(config.location)
		if (!location) continue

		const parts = location.split("/").filter(Boolean)
		let node = root

		for (let index = 0; index < parts.length; index++) {
			const part = parts[index]
			const isLeaf = index === parts.length - 1

			if (isLeaf) {
				node.files.push(config)
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

const getNodeFileCount = (node: TreeNode): number => {
	let count = node.files.length
	for (const child of Object.values(node.folders)) {
		count += getNodeFileCount(child)
	}
	return count
}

const renderTreeNode = (
	node: TreeNode,
	depth: number,
	path: string,
	setSelected: (value: ConfigItem) => void
) => {
	const folderEntries = Object.entries(node.folders).sort(([a], [b]) => a.localeCompare(b))

	return (
		<div className={"w-full"}>
			{folderEntries.length > 0 && (
				<Accordion type={"multiple"} className={"w-full"} defaultValue={folderEntries.map(([folderName]) => `${path}/${folderName}`)}>
					{folderEntries.map(([folderName, childNode]) => {
						const folderPath = path ? `${path}/${folderName}` : folderName
						const fileCount = getNodeFileCount(childNode)

						return (
							<AccordionItem key={folderPath} value={folderPath} className={"border-none"}>
								<AccordionTrigger className={"px-0 py-0 hover:no-underline"}>
									<div className={"flex w-full items-center justify-between gap-3 rounded-lg px-2 py-2 hover:bg-accent/10"} style={{ paddingLeft: depth * 16 }}>
										<div className={"flex min-w-0 items-center gap-3"}>
											<div className={"flex h-8 w-8 items-center justify-center rounded-md border border-border/60 bg-muted/40 text-muted-foreground"}>
												<FolderOpen size={14} className={"shrink-0"} />
											</div>
											<div className={"min-w-0"}>
												<p className={"truncate font-medium"}>{folderName}</p>
											</div>
										</div>

										<div className={"mr-2 flex shrink-0 items-center gap-2"}>
											<Badge variant={"secondary"}>{fileCount}</Badge>
										</div>
									</div>
								</AccordionTrigger>

								<AccordionContent className={"pt-1 pb-2"}>
									<div className={"space-y-1"}>
										{renderTreeNode(childNode, depth + 1, folderPath, setSelected)}
									</div>
								</AccordionContent>
							</AccordionItem>
						)
					})}
				</Accordion>
			)}

			<div className={"space-y-1"}>
				{node.files.map((config) => {
					const location = normalizeLocation(config.location)
					const parts = location.split("/").filter(Boolean)
					const fileName = parts.length > 0 ? parts[parts.length - 1] : location

					return (
						<div
							key={config.uuid}
							role={"button"}
							tabIndex={0}
							onClick={() => setSelected(config)}
							onKeyDown={(event) => {
								if (event.key === "Enter" || event.key === " ") {
									event.preventDefault()
									setSelected(config)
								}
							}}
							className={"group w-full cursor-pointer touch-pan-y rounded-lg px-2 py-2 transition-colors hover:bg-accent/10"}
							style={{ paddingLeft: depth * 16 }}
						>
							<div className={"flex items-center justify-between gap-3"}>
								<div className={"min-w-0 flex items-center gap-3"}>
									<div className={"flex h-8 w-8 items-center justify-center rounded-md border border-border/60 bg-muted/40 text-muted-foreground group-hover:bg-muted"}>
										<FileText size={14} className={"shrink-0"} />
									</div>
									<div className={"min-w-0"}>
										<p className={"truncate font-medium"}>{fileName || config.location}</p>
									</div>
								</div>

								<div className={"shrink-0 flex items-center gap-2"}>
									<Badge variant={"outline"}>{config.format}</Badge>
									<Badge variant={"secondary"}>{config.values} values</Badge>
								</div>
							</div>
						</div>
					)
				})}
			</div>
		</div>
	)
}

export default function PageConfig() {
	const { data: types } = useSWR(
		["types"],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: configs } = useSWR(
		["configs"],
		() => apiGetConfigs(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const [, setSelected] = useState<ConfigItem | null>(null) // 'selected' unused for now; keep setter for future drawer

	const orderedTypes = useMemo<KnownType[]>(
		() => Object.entries(types ?? {}).flatMap(([, group]) => group as KnownType[]),
		[types]
	)

	const typeTrees = useMemo(() => {
		const grouped: Record<string, ConfigItem[]> = {}
		const seenByType: Record<string, Set<string>> = {}

		for (const config of configs ?? []) {
			for (const typeId of getConfigTypeIds(config)) {
				grouped[typeId] = grouped[typeId] ?? []
				seenByType[typeId] = seenByType[typeId] ?? new Set<string>()
				if (seenByType[typeId].has(config.uuid)) continue

				seenByType[typeId].add(config.uuid)
				grouped[typeId].push(config)
			}
		}

		const trees: Record<string, TreeNode> = {}
		for (const [typeId, list] of Object.entries(grouped)) {
			trees[typeId] = buildTree(list)
		}

		return trees
	}, [configs])

	const defaultOpenTypes = useMemo(() => orderedTypes.map((type) => normalizeTypeId(type.identifier)), [orderedTypes])

	return (
		<div>
			<div className={"mb-4 flex items-center justify-between"}>
				<div>
					<h1 className={"text-2xl font-semibold"}>Configs</h1>
					<p className={"text-muted-foreground text-sm"}>{configs ? `${configs.length} configs loaded` : "Loading configs..."}</p>
				</div>
			</div>

			{!configs || !types ? (
				<div className={"w-full"}>
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
					<Skeleton className={"mb-2 h-16 w-full rounded-xl"} />
				</div>
			) : (
				<Accordion type={"multiple"} defaultValue={defaultOpenTypes} className={"w-full"}>
					{orderedTypes.map((type) => {
						const typeId = normalizeTypeId(type.identifier)
						const tree = typeTrees[typeId] ?? { folders: {}, files: [] }
						const fileCount = getNodeFileCount(tree)

						return (
							<Card key={type.identifier} className={"mb-2 overflow-hidden p-0"}>
								<AccordionItem value={typeId} className={"border-none"}>
									<AccordionTrigger className={"px-4 py-0 hover:no-underline"}>
										<div className={"flex w-full items-center justify-between gap-3 mb-2 mt-2"}>
											<div className={"flex min-w-0 items-center gap-3"}>
												<img src={type.icon} alt={type.name} className={"h-10 w-10 rounded-md"} />
												<div className={"min-w-0"}>
													<p className={"truncate font-bold"}>{type.name}</p>
												</div>
											</div>

											<div className={"mr-2 flex shrink-0 items-center gap-2"}>
												<Badge variant={"secondary"}>{fileCount}</Badge>
											</div>
										</div>
									</AccordionTrigger>

									<AccordionContent className={"border-t px-2 py-2"}>
										<div className={"space-y-1"}>
											{renderTreeNode(tree, 0, typeId, setSelected)}
										</div>
									</AccordionContent>
								</AccordionItem>
							</Card>
						)
					})}
				</Accordion>
			)}

			{/*
			<Drawer
				open={Boolean(selected)}
				onOpenChange={(isOpen) => {
					if (!isOpen) setSelected(null)
				}}
				shouldScaleBackground={false}
			>
				<DrawerContent className={"mx-auto w-full max-w-2xl"}>
					<DrawerHeader>
						<DrawerTitle>{selected ? normalizeLocation(selected.location) : "Config details"}</DrawerTitle>
					</DrawerHeader>
					<div className={"space-y-2 px-4 pb-6"}>
						{selected ? (
							<>
								<p className={"text-sm"}><strong>UUID:</strong> {selected.uuid}</p>
								<p className={"text-sm"}><strong>Format:</strong> {selected.format}</p>
								<p className={"text-sm"}><strong>Builds:</strong> {selected.builds}</p>
								<p className={"text-sm"}><strong>Values:</strong> {selected.values}</p>
								<p className={"text-sm"}><strong>Types:</strong> {getConfigTypeIds(selected).join(", ") || "Unknown"}</p>
							</>
						) : null}
					</div>
				</DrawerContent>
			</Drawer>
			*/}
		</div>
	)
}