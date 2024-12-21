import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import useSWR from "swr"
import { JobStatus } from "@/components/job-status"
import { useState } from "react"
import apiGetTypes from "@/api/types"
import { Skeleton } from "@/components/ui/skeleton"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { BarChart3Icon, Building, ChevronDown, CodeIcon, FileIcon, HammerIcon, HomeIcon, LogInIcon, LogOutIcon, SkullIcon, TriangleAlertIcon } from "lucide-react"
import { Link, useLocation } from "react-router-dom"
import { Button } from "@/components/ui/button"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import { useAuth } from "@/hooks/use-auth"
import { BASE_URL } from "@/api"
import apiPostUserLogout from "@/api/user/logout"
import { useToast } from "@/hooks/use-toast"


export function AppSidebar() {
  const [ viewJobs, setViewJobs ] = useState(false)

  const { toast } = useToast()

  const location = useLocation(),
    [user, mutateUser, isUserLoading] = useAuth()

  const { data: types } = useSWR(
    ['types'],
    () => apiGetTypes(),
    { revalidateOnFocus: false, revalidateIfStale: false }
  )

  return (
    <>
      <JobStatus open={viewJobs} onClose={() => setViewJobs(false)} />

      <Sidebar>
        <SidebarHeader>
          <Link to={'/'} className={'flex flex-row h-full w-fit items-center'}>
            <img src={'https://s3.mcjars.app/icons/vanilla.png'} alt={'MCJars'} className={'h-12 w-12'} />

            <div className={'flex flex-col ml-2'}>
              <h1 className={'text-xl font-semibold'}>MCJars</h1>
            </div>
          </Link>
        </SidebarHeader>
        <SidebarContent>
          <Collapsible defaultOpen className={'group/collapsible-information'}>
            <SidebarGroup>
              <SidebarGroupLabel asChild>
                <CollapsibleTrigger>
                  Information
                  <ChevronDown className={'ml-auto transition-transform group-data-[state=open]/collapsible-information:rotate-180'} />
                </CollapsibleTrigger>
              </SidebarGroupLabel>
              <CollapsibleContent>
                <SidebarGroupContent>
                  <SidebarMenu>
                    <SidebarMenuItem>
                      <SidebarMenuButton asChild isActive={location.pathname === '/'}>
                        <Link to={'/'}>
                          <HomeIcon className={'mr-2'} />
                          Home
                        </Link>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                    <SidebarMenuItem>
                      <SidebarMenuButton asChild isActive={location.pathname === '/lookup'}>
                        <Link to={'/lookup'}>
                          <FileIcon className={'mr-2'} />
                          File Lookup
                        </Link>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                    <SidebarMenuItem>
                      <SidebarMenuButton onClick={() => setViewJobs(true)} isActive={viewJobs}>
                        <HammerIcon className={'mr-2'} />
                        Job Status
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                    <SidebarMenuItem>
                      <SidebarMenuButton asChild>
                        <a href={'https://status.mcjars.app'} target={'_blank'} rel={'noreferrer'}>
                          <BarChart3Icon className={'mr-2'} />
                          Status Page
                        </a>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                    <SidebarMenuItem>
                      <SidebarMenuButton asChild>
                        <a href={'https://versions.mcjars.app?warn'} target={'_blank'} rel={'noreferrer'}>
                          <CodeIcon className={'mr-2'} />
                          API Documentation
                        </a>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  </SidebarMenu>
                </SidebarGroupContent>
              </CollapsibleContent>
            </SidebarGroup>
          </Collapsible>

          <Collapsible defaultOpen className={'group/collapsible-builds'}>
            <SidebarGroup>
              <SidebarGroupLabel asChild>
                <CollapsibleTrigger>
                  Types
                  <ChevronDown className={'ml-auto transition-transform group-data-[state=open]/collapsible-builds:rotate-180'} />
                </CollapsibleTrigger>
              </SidebarGroupLabel>
              <CollapsibleContent>
                <SidebarGroupContent>
                  <SidebarMenu>
                    {!types ? (
                      <>
                        <Skeleton className={'mt-2'} />
                      </>
                    ) : (
                      <>
                        {types.map((type) => (
                          <SidebarMenuItem key={type.identifier}>
                            <Collapsible defaultOpen={location.pathname.includes('/'.concat(type.identifier)) || type.identifier === 'VANILLA'} className={'group/collapsible-type'}>
                              <SidebarMenuButton asChild isActive={location.pathname.startsWith(`/${type.name.toUpperCase()}`)}>
                                <CollapsibleTrigger className={'flex flex-row items-center'}>
                                  <img src={type.icon} alt={type.name} className={'h-6 w-6 rounded-md'} />
                                  {type.name}

                                  {type.experimental && (
                                    <Tooltip>
                                      <TooltipTrigger>
                                        <TriangleAlertIcon size={16} className={'text-yellow-500'} />
                                      </TooltipTrigger>
                                      <TooltipContent>
                                        Experimental
                                      </TooltipContent>
                                    </Tooltip>
                                  )}
                                  {type.deprecated && (
                                    <Tooltip>
                                      <TooltipTrigger>
                                        <SkullIcon size={16} className={'text-red-500'} />
                                      </TooltipTrigger>
                                      <TooltipContent>
                                        Deprecated
                                      </TooltipContent>
                                    </Tooltip>
                                  )}

                                  <div className={'ml-auto flex flex-row items-center'}>
                                    <p className={'mr-1'}>({type.builds})</p>

                                    <ChevronDown size={16} className={'transition-transform group-data-[state=open]/collapsible-type:rotate-180'} />
                                  </div>
                                </CollapsibleTrigger>
                              </SidebarMenuButton>
                              <CollapsibleContent>
                                <SidebarMenuSub>
                                  <SidebarMenuSubItem>
                                    <SidebarMenuSubButton asChild isActive={location.pathname === `/${type.name.toUpperCase()}/versions`}>
                                      <Link to={`/${type.name.toUpperCase()}/versions`}>Versions</Link>
                                    </SidebarMenuSubButton>
                                    <SidebarMenuSubButton asChild isActive={location.pathname === `/${type.name.toUpperCase()}/statistics`}>
                                      <Link to={`/${type.name.toUpperCase()}/statistics`}>Statistics</Link>
                                    </SidebarMenuSubButton>
                                  </SidebarMenuSubItem>
                                </SidebarMenuSub>
                              </CollapsibleContent>
                            </Collapsible>
                          </SidebarMenuItem>
                        ))}
                      </>
                    )}
                  </SidebarMenu>
                </SidebarGroupContent>
              </CollapsibleContent>
            </SidebarGroup>
          </Collapsible>
        </SidebarContent>
        <SidebarFooter>
          {isUserLoading ? (
            <Button className={'w-full flex-row items-center justify-between'} variant={'secondary'}>
              <Skeleton className={'h-6 w-6 rounded-full'} />
              <Skeleton className={'h-6 w-20'} />
            </Button>
          ) : (
            user ? (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button className={'w-full flex-row items-center justify-between'} variant={'secondary'}>
                    <img src={user.avatar} alt={user.name ?? `@${user.login}`} className={'h-6 w-6 rounded-full'} />
                    <span className={'ml-2 truncate'}>{user.name ?? `@${user.login}`}</span>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align={'end'}>
                  <DropdownMenuItem asChild>
                    <Link to={'/organizations'} className={'w-full'}>
                      <Building size={24} />
                      Organizations
                    </Link>
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={() => {
                    toast({
                      title: 'Logging out...',
                      description: 'You are being logged out of MCJars.'
                    })

                    apiPostUserLogout().then(() => {
                      toast({
                        title: 'Logged out',
                        description: 'You have been logged out of MCJars.'
                      })

                      mutateUser(null, false)
                    })
                  }}>
                    <LogOutIcon size={24} />
                    Log Out
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            ) : (
              <a href={`${BASE_URL}/api/github`} className={'w-full'} onClick={() => {
                toast({
                  title: 'Logging in...',
                  description: 'You are being redirected to GitHub to login to MCJars.'
                })
              }}>
                <Button className={'w-full flex-row items-center justify-between'} variant={'secondary'}>
                  <LogInIcon size={24} />
                  <span className={'ml-2'}>Login</span>
                </Button>
              </a>
            )
          )}
        </SidebarFooter>
      </Sidebar>
    </>
  )
}
