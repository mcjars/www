import { Button } from "@/components/ui/button"
import { useEffect, useState } from "react"
import useSWR from "swr"
import apiGetTypes from "@/api/types"
import apiGetVersions from "@/api/versions"
import apiGetBuilds, { PartialMinecraftBuild } from "@/api/builds"
import apiGetBuild from "@/api/build"
import apiGetStats from "@/api/stats"
import apiGetConfig from "@/api/config"
import { Skeleton } from "@/components/ui/skeleton"
import { BooleanParam, StringParam, useQueryParam } from "use-query-params"
import bytes from "bytes"
import { Drawer, DrawerContent } from "@/components/ui/drawer"
import { cn } from "@/lib/utils"
import { TbArchiveFilled, TbArrowLeft, TbArrowRight, TbBrandGithub, TbDownload, TbExternalLink, TbHammer, TbLink, TbTrashFilled } from "react-icons/tb"
import { FoliaFlowchart } from "@/components/folia-flowchart"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer"

export default function App() {
  const [ includeSnapshots, setIncludeSnapshots ] = useQueryParam('snapshots', BooleanParam)
  const [ includeExperimental, setIncludeExperimental ] = useQueryParam('experimental', BooleanParam)
  const [ type, setType ] = useQueryParam('type', StringParam)
  const [ version, setVersion ] = useQueryParam('version', StringParam)
  const [ build, setBuild ] = useState<PartialMinecraftBuild>()
  const [ isDragging, setIsDragging ] = useState(false)
  const [ isDropLoading, setIsDropLoading ] = useState(false)
  const [ jarDropBuild, setJarDropBuild ] = useState<PartialMinecraftBuild>()
  const [ configDropMatches, setConfigDropMatches ] = useState<Awaited<ReturnType<typeof apiGetConfig>>>()
  const [ configDropMatch, setConfigDropMatch ] = useState<Awaited<ReturnType<typeof apiGetConfig>>['configs'][number]>()
  const [ step, setStep ] = useState(0)

  const { data: types } = useSWR(
    ['types'],
    () => apiGetTypes(),
    { revalidateOnFocus: false, revalidateIfStale: false }
  )

  const { data: stats } = useSWR(
    ['stats'],
    () => apiGetStats(),
    { revalidateOnFocus: false, revalidateIfStale: false }
  )

  const { data: versions, isValidating: validatingVersions } = useSWR(
    ['versions', type],
    () => type ? apiGetVersions(type) : undefined,
    { revalidateOnFocus: false, revalidateIfStale: false }
  )

  const { data: builds, isValidating: validatingBuilds } = useSWR(
    ['builds', type, version],
    () => type && version ? apiGetBuilds(type, version) : undefined,
    { revalidateOnFocus: false, revalidateIfStale: false }
  )

  useEffect(() => {
    if (build) {
      setStep(1)
    } else {
      setStep(0)
    }
  }, [ build ])

  useEffect(() => {
    if (configDropMatches) {
      setConfigDropMatch(configDropMatches.configs[0])
    }
  }, [ configDropMatches ])

  useEffect(() => {
    if (types && !type) {
      setType(types[0].identifier)
    }
  }, [ types ])

  useEffect(() => {
    if (versions && !versions.find((v) => (v.latest.versionId ?? v.latest.projectVersionId) === version)) {
      const index = versions.findIndex((v) => v.type === 'RELEASE' || !v.type)

      setVersion(versions[index].latest.versionId ?? versions[index].latest.projectVersionId)
    }
  }, [ versions, version ])

  useEffect(() => {
    window.addEventListener('dragenter', (e) => {
      e.preventDefault()
      setIsDragging(true)
    })

    window.addEventListener('dragover', (e) => {
      e.preventDefault()
      setIsDragging(true)
    })

    window.addEventListener('dragleave', (e) => {
      e.preventDefault()
      setIsDragging(false)
    })

    window.addEventListener('drop', async(e) => {
      e.preventDefault()
      setIsDragging(false)

      const file = e.dataTransfer?.files[0]
      if (!file) return

      if (file.name.endsWith('.jar')) {
        setIsDropLoading(true)

        const hash = await crypto.subtle.digest('SHA-256', new Uint8Array(await file.arrayBuffer())),
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
            buildNumber: 1,
            changes: [],
            created: null,
            experimental: false,
            id: 1,
            installation: [],
            jarLocation: null,
            jarSize: null,
            jarUrl: null,
            zipSize: null,
            zipUrl: null,
            projectVersionId: null,
            versionId: 'Unknown'
          })
        }
      } else if (
        file.name.endsWith('.yml') ||
        file.name.endsWith('.properties') ||
        file.name.endsWith('.toml') ||
        file.name.endsWith('.conf')
      ) {
        setIsDropLoading(true)

        const config = await apiGetConfig(file)

        setIsDropLoading(false)
        setConfigDropMatches(config)
      }
    })
  }, [])

  return (
    <>
      <Dialog open={Boolean(configDropMatches)} onOpenChange={(open) => setConfigDropMatches((c) => open ? c : undefined)}>
        {configDropMatches && (
          <DialogContent className={'w-[50vw] max-w-[50vw] h-[75vh]'}>
            <DialogHeader>
              <DialogTitle className={'flex flex-row w-full justify-between'}>
                {configDropMatches.configs.map((config) => (
                  <Button disabled={config === configDropMatch} onClick={() => setConfigDropMatch(config)} key={config.build?.id ?? config.from} className={'flex flex-row justify-start items-center w-full mr-4'}>
                    <img src={types?.find((t) => t.identifier === config.from)?.icon} alt={config.from ?? undefined} className={'h-6 w-6 mr-2 rounded-md'} />
                    <span className={'flex flex-col'}>
                      <p className={'text-left'}>{config.from}</p>
                      <p className={'text-xs -mt-1'}>{config.build?.versionId} #{config.build?.buildNumber}</p>
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

      <Drawer open={isDragging || isDropLoading || Boolean(jarDropBuild)} onOpenChange={(open) => {
        if (isDropLoading) return

        setIsDragging(open)

        if (!open) {
          setJarDropBuild(undefined)
        }
      }}>
        <DrawerContent className={'w-full max-w-3xl mx-auto'}>
          {!isDropLoading && !jarDropBuild ? (
            <div className={'flex flex-col items-center justify-center h-full'}>
              <h1 className={'text-2xl font-semibold'}>Drop Jar File or Config File</h1>
              <p className={'text-xs'}>Drop the Jar or Config file to check what build and type it is.</p>
            </div>
          ) : jarDropBuild ? (
            <div className={'flex flex-row justify-between items-center p-2'}>
              <div className={'flex flex-row'}>
                <img src={types?.find((t) => t.identifier === jarDropBuild.type)?.icon ?? 'https://s3.mcjars.app/icons/vanilla.png'} alt={jarDropBuild.type ?? undefined} className={'h-24 w-24 mr-2 rounded-md'} />
                <div className={'flex flex-col items-start'}>
                  <h1 className={'text-xl font-semibold'}>{types?.find((t) => t.identifier === jarDropBuild.type)?.name ?? 'Unknown'}</h1>
                  {jarDropBuild.buildNumber === 1 && jarDropBuild.projectVersionId ? <h1 className={'text-xl'}>{`Version ${jarDropBuild.projectVersionId}`}</h1> : <h1 className={'text-md'}>{`Build #${jarDropBuild.buildNumber}`}</h1>}
                  <p>{bytes(jarDropBuild.jarSize ?? jarDropBuild.zipSize ?? 0)}</p>
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

      <Drawer open={Boolean(build)} onOpenChange={(open) => setBuild(open ? build : undefined)}>
        <DrawerContent className={'w-full max-w-xl mx-auto h-fit'}>
          {build && (
            <div className={'flex flex-row justify-between items-center p-2'}>
              <img src={types?.find((t) => t.identifier === type)?.icon} alt={type ?? undefined} className={'h-24 w-24 mr-2 rounded-md'} />
              <span className={'flex flex-col h-full w-full'}>
                <h1 className={'font-semibold text-xl'}>Installation</h1>
                <code
                  className={'mt-1 w-fit select-text md:block hidden text-xs hover:font-semibold cursor-pointer'}
                  onClick={() => navigator.clipboard.writeText(`bash <(curl -s ${window.location.protocol}//${window.location.hostname}/install.sh) ${build.id}`)}
                >
                  bash &lt;(curl -s {window.location.protocol}//{window.location.hostname}/install.sh) {build.id}
                </code>
                <div className={'mb-1.5 mt-5 flex flex-row items-center'}>
                  <Popover modal>
                    <PopoverTrigger className={'w-full h-full mr-1'}>
                      <Button className={'mt-auto mb-0 w-full h-full'}>
                        <TbHammer size={24} className={'mr-1'} />
                        <span className={'flex flex-col items-center'}>
                          <p className={'font-semibold'}>Install</p>
                          <p className={'text-xs -mt-1'}>{build.installation.reduce((a, b) => a + b.length, 0)} Steps</p>
                        </span>
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent align={'end'} className={'w-80 h-40'}>
                      <div className={'flex flex-row'}>
                        <h1 className={'font-semibold my-auto'}>Step {step}</h1>
                        {build.installation.reduce((a, b) => a + b.length, 0) > 1 && (
                          <div className={'flex flex-row ml-auto'}>
                            <Button className={'mr-2'} disabled={step === 1} onClick={() => setStep(step - 1)}>
                              <TbArrowLeft size={24} className={'mr-1'} />
                              Previous
                            </Button>
                            <Button disabled={step === build.installation.reduce((a, b) => a + b.length, 0)} onClick={() => setStep(step + 1)}>
                              <TbArrowRight size={24} className={'mr-1'} />
                              Next
                            </Button>
                          </div>
                        )}
                      </div>

                      {((s = build.installation.flat().at(step - 1)) => s && (
                        <div className={'w-full h-full grid grid-cols-4 mt-4'}>
                          <span className={'col-span-1 mx-auto'}>
                            {s.type === 'download'
                              ? <TbDownload size={64} />
                              : s.type === 'unzip'
                                ? <TbArchiveFilled size={64} />
                                : <TbTrashFilled size={64} />}
                          </span>
                          {s.type === 'download' && (
                            <span className={'col-span-3 flex flex-col'}>
                              <h1 className={'font-semibold'}>Download</h1>
                              <p className={'text-sm'}>Download <a href={s.url} className={'text-blue-500'}>this</a> as <code>{s.file}</code></p>
                            </span>
                          )}
                          {s.type === 'unzip' && (
                            <span className={'col-span-3 flex flex-col'}>
                              <h1 className={'font-semibold'}>Unzip</h1>
                              <p className={'text-sm'}>Unzip <code>{s.file}</code> in <code>{s.location}</code></p>
                            </span>
                          )}
                          {s.type === 'remove' && (
                            <span className={'col-span-3 flex flex-col'}>
                              <h1 className={'font-semibold'}>Delete</h1>
                              <p className={'text-sm'}>Delete <code>{s.location}</code></p>
                            </span>
                          )}
                        </div>
                      ))()}
                    </PopoverContent>
                  </Popover>
                  {build.changes.length > 0 && (
                    <Popover modal>
                      <PopoverTrigger className={'w-full h-full'}>
                        <Button className={'w-full h-full'}>
                          <TbExternalLink size={24} className={'mr-1'} />
                          <span className={'flex flex-col items-center'}>
                            <p className={'font-semibold'}>View</p>
                            <p className={'text-xs -mt-1'}>{build.changes.length} Change{build.changes.length === 1 ? '' : 's'}</p>
                          </span>
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
                </div>
              </span>
            </div>
          )}
        </DrawerContent>
      </Drawer>

      <FoliaFlowchart open={type?.toUpperCase() === 'FOLIA'} onClose={() => setType('PAPER')} />

      <nav className={'flex flex-row items-center justify-between px-4 py-2 border-b-2 border-x-2 rounded-b-xl w-full max-w-7xl h-16 mx-auto'}>
        <div className={'flex flex-row h-full items-center'}>
          <img src={'https://s3.mcjars.app/icons/vanilla.png'} alt={'Logo'} className={'h-12 w-12'} />
          <div className={'flex flex-col ml-2'}>
            <h1 className={'text-xl font-semibold'}>MCJars</h1>
            {stats && (
              <p className={'text-xs -mt-1'}>{stats.builds} Total Builds, {stats.hashes} Hashes</p>
            )}
          </div>
        </div>
        <p className={'xl:block hidden text-xs'}>
          You can drag in your server jar to detect it!
        </p>
        <div className={'md:flex hidden space-x-1 flex-row h-full items-center'}>
          <a href={'https://versions.mcjars.app'} target={'_blank'} rel={'noopener noreferrer'}>
            <Button>
              <TbLink size={24} className={'mr-1'} />
              API Docs
            </Button>
          </a>
          <a href={'https://github.com/mcjars'} target={'_blank'} rel={'noopener noreferrer'}>
            <Button>
              <TbBrandGithub size={24} className={'mr-1'} />
              GitHub
            </Button>
          </a>
        </div>
      </nav>
      <main className={'p-4 pt-0 grid xl:grid-cols-8 xl:grid-rows-1 grid-rows-8 gap-2 w-full h-[calc(100vh-5rem)] max-w-7xl mx-auto'}>
        <div className={'flex flex-col xl:col-span-3 xl:row-span-1 row-span-3 overflow-scroll xl:h-[calc(100vh-5rem)]'}>
          {types ? (
            <>
              {types.map((t) => (
                <Button
                  key={t.identifier}
                  disabled={t.identifier === type}
                  onClick={() => setType(t.identifier)}
                  className={'h-fit my-1 flex flex-row items-center justify-between w-full text-right'}
                >
                  <img src={t.icon} alt={t.name} className={'h-16 w-16 mr-2 rounded-md'} />
                  <span>
                    <h1 className={'md:text-xl text-lg font-semibold'}>{t.name}</h1>
                    <p className={'mb-[6px]'}>
                      {t.categories.map((c) => (
                        <span key={t.name + c} className={'-md:hidden text-xs mr-1 bg-blue-500 text-white h-6 p-1 rounded-md'}>{c}</span>
                      ))}
                      {t.experimental && <span className={'text-xs mr-1 bg-yellow-500 text-white h-6 p-1 rounded-md'}>experimental</span>}
                      {t.deprecated && <span className={'text-xs mr-1 bg-red-500 text-white h-6 p-1 rounded-md'}>deprecated</span>}
                      {t.builds} Build{t.builds === 1 ? '' : 's'}
                    </p>
                    <span className={'md:block hidden'}>
                      {t.compatibility.map((c) => (
                        <span key={t.name + c} className={'text-xs mr-1 bg-green-500 text-white h-6 p-1 rounded-md'}>{c}</span>
                      ))}
                      {t.compatibility.length > 0 && 'compatibility'}
                    </span>
                  </span>
                </Button>
              ))}
            </>
          ) : (
            <>
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
            </>
          )}
        </div>
        <div className={'flex flex-col xl:col-span-2 xl:row-span-1 row-span-2 overflow-scroll xl:h-[calc(100vh-5rem)]'}>
          {!validatingVersions && versions && types ? (
            <>
              {versions.some((v) => v.type === 'SNAPSHOT') && (
                <Button
                  onClick={() => setIncludeSnapshots(!includeSnapshots)}
                  className={cn('my-1', includeSnapshots ? 'bg-green-500 hover:bg-green-400' : 'bg-red-500 hover:bg-red-400')}
                >
                  Include Snapshots
                </Button>
              )}
              {versions.filter((v) => !v.type || (v.latest.versionId ?? v.latest.projectVersionId) === version || v.type === 'RELEASE' || (v.type === 'SNAPSHOT' && includeSnapshots)).map((v) => (
                <Button
                  key={v.latest.versionId ?? v.latest.projectVersionId}
                  disabled={(v.latest.versionId ?? v.latest.projectVersionId) === version}
                  onClick={() => setVersion(v.latest.versionId ?? v.latest.projectVersionId)}
                  className={'h-16 my-1 flex flex-row items-center justify-between w-full text-right'}
                >
                  <img src={types.find((t) => t.identifier === type)?.icon} alt={type ?? undefined} className={'h-12 w-12 mr-2 rounded-md'} />
                  <span>
                    <h1 className={'md:text-xl text-lg font-semibold'}>{v.latest.versionId ?? v.latest.projectVersionId}</h1>
                    <span className={'grid grid-cols-2 mr-0'}>
                      <p>{v.builds} Build{v.builds === 1 ? '' : 's'}</p>
                      <p className={'w-fit text-right pl-2'}>Requires Java {v.java}</p>
                    </span>
                  </span>
                </Button>
              ))}
            </>
          ) : (
            <>
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
            </>
          )}
        </div>
        <div className={'flex flex-col xl:col-span-3 xl:row-span-1 row-span-3 overflow-scroll xl:h-[calc(100vh-5rem)]'}>
          {!validatingBuilds && builds && versions && types ? (
            <>
              {builds.some((b) => b.experimental) && !builds.every((b) => b.experimental) && (
                <Button
                  onClick={() => setIncludeExperimental(!includeExperimental)}
                  className={cn('my-1', includeExperimental ? 'bg-green-500 hover:bg-green-400' : 'bg-red-500 hover:bg-red-400')}
                >
                  Include Experimental
                </Button>
              )}
              {builds.filter((b) => b.experimental && !builds.every((b) => b.experimental) ? includeExperimental : true).map((b) => (
                <Button
                  key={b.id}
                  disabled={b.id === build?.id}
                  onClick={() => setBuild(b)}
                  className={'h-fit my-1 flex flex-row items-center justify-between w-full text-right'}
                >
                  <img src={types.find((t) => t.identifier === type)?.icon} alt={type ?? undefined} className={'h-12 w-12 mr-2 rounded-md'} />
                  <span>
                    <h1 className={'md:text-xl text-lg font-semibold'}>{b.buildNumber === 1 && b.projectVersionId ? b.projectVersionId : `Build #${b.buildNumber}`}</h1>
                    <p className={'mb-[2px]'}>
                      {b.experimental
                        ? <span className={'text-xs mr-1 bg-red-500 text-white h-6 p-1 rounded-md'}>experimental</span>
                        : <span className={'text-xs mr-1 bg-green-500 text-white h-6 p-1 rounded-md'}>stable</span>
                      }

                      {bytes(b.installation.flat().filter((i) => i.type === 'download').reduce((a, b) => a + b.size, 0))}
                      {b.changes.length > 0 && ` ${b.changes.length} Change${b.changes.length === 1 ? '' : 's'}`}
                    </p>
                    <span>
                      <p>{b.created}</p>
                    </span>
                  </span>
                </Button>
              ))}
            </>
          ) : (
            <>
              <Skeleton className={'h-16 my-1'} />
              <Skeleton className={'h-16 my-1'} />
            </>
          )}
        </div>
      </main>
    </>
  )
}