import React from "react"
import ReactDOM from "react-dom/client"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryParamProvider } from "use-query-params"
import { ReactRouter6Adapter } from "use-query-params/adapters/react-router-6"
import { BrowserRouter, Route, Routes } from "react-router-dom"
import { AppSidebar } from "@/components/app-sidebar"
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { Toaster } from "@/components/ui/toaster"

import Page404 from "@/pages/404"
import PageIndex from "@/pages/index"
import PageLookup from "@/pages/lookup"
import PageJobStatus from "@/pages/job-status"
import PageOrganizations from "@/pages/organizations"
import PageTypeVersions from "@/pages/{type}/versions"
import PageTypeStatistics from "@/pages/{type}/statistics"

import "@/global.css"

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <QueryParamProvider adapter={ReactRouter6Adapter}>
        <ThemeProvider defaultTheme={'dark'}>
          <SidebarProvider>
            <AppSidebar />
            <main className={'relative h-screen md:w-[calc(100vw-var(--sidebar-width))] w-screen md:pt-2 pt-14 px-2'}>
              <SidebarTrigger className={'left-2 top-2 absolute md:hidden'} />
              <Routes>
                <Route path={'/'} element={<PageIndex />} />
                <Route path={'/lookup'} element={<PageLookup />} />
                <Route path={'/job-status'} element={<PageJobStatus />} />
                <Route path={'/organizations'} element={<PageOrganizations />} />
                <Route path={'/:type/versions'} element={<PageTypeVersions />} />
                <Route path={'/:type/statistics'} element={<PageTypeStatistics />} />
                <Route path={'*'} element={<Page404 />} />
              </Routes>
              <Toaster />
            </main>
          </SidebarProvider>
        </ThemeProvider>
      </QueryParamProvider>
    </BrowserRouter>
  </React.StrictMode>
)