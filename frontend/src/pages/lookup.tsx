import apiGetBuild from "@/api/build"
import { PartialMinecraftBuild } from "@/api/builds"
import apiGetConfig from "@/api/config"
import apiGetTypes from "@/api/types"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Drawer, DrawerContent } from "@/components/ui/drawer"
import { Skeleton } from "@/components/ui/skeleton"
import bytes from "bytes"
import { LoaderCircle } from "lucide-react"
import { LegacyRef, useEffect, useMemo, useRef, useState } from "react"
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer"
import useSWR from "swr"

export default function PageLookup() {
	const [ isDragging, setIsDragging ] = useState(false)
	const [ isDropLoading, setIsDropLoading ] = useState(false)
	const [ jarDropBuild, setJarDropBuild ] = useState<PartialMinecraftBuild>()
	const [ configDropMatches, setConfigDropMatches ] = useState<Awaited<ReturnType<typeof apiGetConfig>>>()
	const [ configDropMatch, setConfigDropMatch ] = useState<Awaited<ReturnType<typeof apiGetConfig>>['configs'][number]>()
	const inputRef = useRef<HTMLInputElement>()

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
					type: 'UNKNOWN',
					name: '???',
					changes: [],
					created: null,
					experimental: false,
					id: 1,
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

			const config = await apiGetConfig(file)

			setIsDropLoading(false)
			setConfigDropMatches(config)
		}
	}

	useEffect(() => {
		if (configDropMatches) {
			setConfigDropMatch(configDropMatches.configs[0])
		}
	}, [ configDropMatches ])

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
		[ types, jarDropBuild ]
	)

	return (
		<>
			<input
				id={'file-input'}
				type={'file'}
				className={'hidden'}
				ref={inputRef as LegacyRef<HTMLInputElement>}
				accept={'.jar,.yml,.properties,.toml,.conf'}
				onChange={(e) => {
					const file = e.target.files?.[0]
					if (!file) return

					handleFile(file)
				}}
			/>

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

			<Dialog open={Boolean(configDropMatches)} onOpenChange={(open) => setConfigDropMatches((c) => open ? c : undefined)}>
        {configDropMatches && (
          <DialogContent className={'w-[50vw] max-w-[50vw] h-[75vh]'}>
            <DialogHeader>
              <DialogTitle className={'flex flex-row w-full justify-between'}>
                {configDropMatches.configs.map((config) => (
                  <Button disabled={config === configDropMatch} onClick={() => setConfigDropMatch(config)} key={config.build?.id ?? config.from} className={'flex flex-row justify-start items-center w-full mr-4'}>
                    <img src={Object.values(types ?? {}).flat().find((t) => t.identifier === config.from)?.icon} alt={config.from ?? undefined} className={'h-6 w-6 mr-2 rounded-md'} />
                    <span className={'flex flex-col'}>
                      <p className={'text-left'}>{config.from}</p>
                      <p className={'text-xs -mt-1'}>{config.build?.versionId} {config.build?.name}</p>
                    </span>
                  </Button>
                ))}
              </DialogTitle>
              <DialogDescription className={'h-full relative overflow-scroll rounded-md'}>
                {configDropMatch && (
                  <ReactDiffViewer
                    splitView
                    useDarkTheme
                    showDiffOnly={false}
                    oldValue={configDropMatches.formatted}
                    newValue={configDropMatch.value}
                    compareMethod={DiffMethod.LINES}
                    leftTitle={'Original'}
                    rightTitle={'Remote Match'}
                    styles={{
                      diffContainer: {
                        position: 'absolute',
                        height: '100%'
                      }
                    }}
                  />
                )}
              </DialogDescription>
            </DialogHeader>
          </DialogContent>
        )}
      </Dialog>

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