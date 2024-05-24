import { Button } from "@/components/ui/button"
import { useEffect, useState } from "react"
import useSWR from "swr"
import apiGetTypes from "@/api/types"
import apiGetVersions from "@/api/versions"
import apiGetBuilds, { PartialMinecraftBuild } from "@/api/builds"
import apiGetBuild from "@/api/build"
import { Skeleton } from "@/components/ui/skeleton"
import { BooleanParam, StringParam, useQueryParam } from "use-query-params"
import bytes from "bytes"
import { Drawer, DrawerContent } from "@/components/ui/drawer"
import { cn } from "@/lib/utils"
import { TbDownload } from "react-icons/tb"

export default function App() {
  const [ includeSnapshots, setIncludeSnapshots ] = useQueryParam('snapshots', BooleanParam)
  const [ type, setType ] = useQueryParam('type', StringParam)
  const [ version, setVersion ] = useQueryParam('version', StringParam)
  const [ build, setBuild ] = useState<PartialMinecraftBuild>()
  const [ isDragging, setIsDragging ] = useState(false)
  const [ isJarDropLoading, setIsJarDropLoading ] = useState(false)
  const [ jarDropBuild, setJarDropBuild ] = useState<PartialMinecraftBuild>()

  const { data: types } = useSWR(
    ['types'],
    () => apiGetTypes(),
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
    if (types && !type) {
      setType(types[0].identifier)
    }
  }, [ types ])

  useEffect(() => {
    if (versions && !versions.find((v) => (v.latest.versionId ?? v.latest.projectVersionId) === version)) {
      setVersion(versions[0].latest.versionId ?? versions[0].latest.projectVersionId)
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

    window.addEventListener('drop', (e) => {
      e.preventDefault()
      setIsDragging(false)
      setIsJarDropLoading(true)

      const file = e.dataTransfer?.files[0]
      if (!file) return

      const reader = new FileReader()

      reader.onload = async() => {
        setIsJarDropLoading(true)
        const hash = await crypto.subtle.digest('SHA-256', new Uint8Array(reader.result as ArrayBuffer))
        const hashArray = Array.from(new Uint8Array(hash))
        const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')

        try {
          const build = await apiGetBuild(hashHex)
          setJarDropBuild(build)
          setIsJarDropLoading(false)
        } catch {
          setIsJarDropLoading(false)
        }
      }

      reader.readAsArrayBuffer(file)
    })
  }, [])

  return (
    <>
      <Drawer open={isDragging || isJarDropLoading || Boolean(jarDropBuild)} onOpenChange={(open) => {
        if (isJarDropLoading) return

        setIsDragging(open)

        if (!open) {
          setJarDropBuild(undefined)
        }
      }}>
        <DrawerContent className={'w-full max-w-3xl mx-auto'}>
          {!isJarDropLoading && !jarDropBuild ? (
            <div className={'flex flex-col items-center justify-center h-full'}>
              <h1 className={'text-2xl font-semibold'}>Drop Jar File</h1>
              <p className={'text-xs'}>Drop the Jar file to check what build and type it is.</p>
            </div>
          ) : jarDropBuild ? (
            <div className={'flex flex-row justify-between items-center p-2'}>
              <div className={'flex flex-row'}>
                <img src={types?.find((t) => t.identifier === jarDropBuild.type)?.icon} alt={jarDropBuild.type ?? undefined} className={'h-24 w-24 mr-2 rounded-md'} />
                <div className={'flex flex-col items-start'}>
                  <h1 className={'text-xl font-semibold'}>{types?.find((t) => t.identifier === jarDropBuild.type)?.name}</h1>
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
        <DrawerContent className={'w-full max-w-3xl mx-auto'}>
          {build && (
            <div className={'flex flex-row justify-between items-center p-2'}>
              <img src={types?.find((t) => t.identifier === type)?.icon} alt={type ?? undefined} className={'h-24 w-24 mr-2 rounded-md'} />
              <span className={'text-left w-96 self-start'}>
                <h1 className={'font-semibold text-xl'}>Installation</h1>
                {build.zipUrl && (
                  <>
                    <p className={'text-xs'}>Download the zip file and extract it to your server's root folder.</p>
                    {build.jarUrl && (
                      <>
                        <p className={'text-xs flex flex-row'}>Download the Jar file and check for a <p className={'mx-1 font-bold'}>.mcvapi.jarUrl.txt</p> file.</p>
                        <p className={'ml-2 text-xs'}>If the file exists, rename the Jar file to the value inside the file.</p>
                        <p className={'ml-2 text-xs flex flex-row'}>If the file does not exist, rename the Jar file to <p className={'ml-1 font-bold'}>server.jar</p>.</p>
                      </>
                    )}
                    <p className={'text-xs flex flex-row'}>The Jar for starting the server will be <p className={'ml-1 font-bold'}>{build.zipUrl?.split('/').pop()?.slice(0, -4)}</p>.</p>
                  </>
                )}
                {build.jarUrl && !build.zipUrl && (
                  <>
                    <p className={'text-xs'}>Download the Jar file and place it in your server's root folder.</p>
                    <p className={'text-xs flex flex-row'}>Rename the Jar file to <p className={'ml-1 font-bold'}>server.jar</p>.</p>
                    <p className={'text-xs flex flex-row'}>The Jar for starting the server will be <p className={'ml-1 font-bold'}>server.jar</p>.</p>
                  </>
                )}
              </span>
              <div className={'flex flex-col items-center w-48 space-y-1 h-full'}>
                {build.jarUrl && (
                  <a href={build.jarUrl ?? undefined} target={'_blank'} rel={'noopener noreferrer'}>
                    <Button className={'w-full h-full'}>
                      <TbDownload size={24} className={'mr-1'} />
                      <span className={'flex flex-col items-center'}>
                        <p className={'font-semibold'}>Download Jar</p>
                        <p className={'text-xs -mt-1'}>{bytes(build.jarSize ?? 0)}</p>
                      </span>
                    </Button>
                  </a>
                )}
                {build.zipUrl && (
                  <a href={build.zipUrl ?? undefined} target={'_blank'} rel={'noopener noreferrer'}>
                    <Button className={'w-full h-full'}>
                      <TbDownload size={24} className={'mr-1'} />
                      <span className={'flex flex-col items-center'}>
                        <p className={'font-semibold'}>Download Zip</p>
                        <p className={'text-xs -mt-1'}>{bytes(build.zipSize ?? 0)}</p>
                      </span>
                    </Button>
                  </a>
                )}
              </div>
            </div>
          )}
        </DrawerContent>
      </Drawer>

      <main className={'p-4 grid xl:grid-cols-8 xl:grid-rows-1 grid-rows-8 gap-2 w-full h-[calc(100vh-2rem)] max-w-7xl mx-auto'}>
        <div className={'flex flex-col xl:col-span-3 xl:row-span-1 row-span-3 overflow-scroll pr-4 xl:h-[calc(100vh-2rem)]'}>
          {types ? (
            <>
              {types.map((t) => (
                <Button
                  key={t.identifier}
                  disabled={t.identifier === type}
                  onClick={() => setType(t.identifier)}
                  className={'h-16 my-1 flex flex-row items-center justify-between w-full text-right'}
                >
                  <img src={t.icon} alt={t.name} className={'h-12 w-12 mr-2 rounded-md'} />
                  <span>
                    <h1 className={'text-xl font-semibold'}>{t.name}</h1>
                    <p>{t.builds} Build{t.builds === 1 ? '' : 's'}</p>
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
        <div className={'flex flex-col xl:col-span-2 xl:row-span-1 row-span-2 overflow-scroll pr-4 xl:h-[calc(100vh-2rem)]'}>
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
                    <h1 className={'text-xl font-semibold'}>{v.latest.versionId ?? v.latest.projectVersionId}</h1>
                    <p>{v.builds} Build{v.builds === 1 ? '' : 's'}</p>
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
        <div className={'flex flex-col xl:col-span-3 xl:row-span-1 row-span-3 overflow-scroll pr-4 xl:h-[calc(100vh-2rem)]'}>
          {!validatingBuilds && builds && versions && types ? (
            <>
              {builds.map((b) => (
                <Button
                  key={b.id}
                  disabled={b.id === build?.id}
                  onClick={() => setBuild(b)}
                  className={'h-16 my-1 flex flex-row items-center justify-between w-full text-right'}
                >
                  <img src={types.find((t) => t.identifier === type)?.icon} alt={type ?? undefined} className={'h-12 w-12 mr-2 rounded-md'} />
                  <span>
                    <h1 className={'text-xl font-semibold'}>{b.buildNumber === 1 && b.projectVersionId ? `Version ${b.projectVersionId}` : `Build #${b.buildNumber}`}</h1>
                    <span className={'grid w-60 grid-cols-2'}>
                      <p>{b.created}</p>
                      <p>{bytes(b.jarSize ?? b.zipSize ?? 0)}</p>
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
      </main>
    </>
  )
}