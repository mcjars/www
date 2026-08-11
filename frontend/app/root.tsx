import { QueryClientProvider } from '@tanstack/react-query';
import { Links, Meta, Outlet, Scripts, ScrollRestoration, isRouteErrorResponse } from 'react-router';

import { AppSidebar } from '~/components/app-sidebar.tsx';
import { ThemeProvider } from '~/components/theme-provider.tsx';
import { SidebarProvider, SidebarTrigger } from '~/components/ui/sidebar.tsx';
import { Toaster } from '~/components/ui/toaster.tsx';
import { getQueryClient } from '~/lib/query-client.ts';

import type { Route } from './+types/root';

import './app.css';

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta name="darkreader-lock" />
        <meta property="og:type" content="website" />
        <meta property="og:site_name" content="MCJars" />
        <meta name="twitter:card" content="summary" />
        <meta
          name="keywords"
          content="minecraft, server, jar, download, lookup, reverse, lookup, mcjars, site, spigot download, latest version, 1.21, server jar"
        />
        <link rel="icon" type="image/png" href="https://s3.mcjars.app/icons/vanilla.png" />
        <script defer data-domain="mcjars.app" src="https://cat.rjns.dev/js/script.js" />
        <Meta />
        <Links />
      </head>
      <body>
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export function meta(): Route.MetaDescriptors {
  return [{ title: 'MCJars' }];
}

export default function App() {
  const queryClient = getQueryClient();

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider defaultTheme={'dark'}>
        <SidebarProvider>
          <AppSidebar />
          <main
            className={'relative h-screen md:w-[calc(100vw-var(--sidebar-width))] w-screen md:pt-2 pt-14 px-2'}
          >
            <SidebarTrigger className={'left-2 top-2 absolute md:hidden'} />
            <Outlet />
            <Toaster />
          </main>
        </SidebarProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  let message = 'Oops!';
  let details = 'An unexpected error occurred.';
  let stack: string | undefined;

  if (isRouteErrorResponse(error)) {
    message = error.status === 404 ? '404' : 'Error';
    details = error.status === 404 ? 'The requested page could not be found.' : (error.statusText ?? details);
  } else if (import.meta.env.DEV && error instanceof Error) {
    details = error.message;
    ({ stack } = error);
  }

  return (
    <main className={'flex min-h-screen flex-col items-center justify-center gap-4 p-4'}>
      <h1 className={'text-4xl font-bold'}>{message}</h1>
      <p className={'text-muted-foreground'}>{details}</p>
      {stack && <pre className={'max-w-xl overflow-x-auto rounded-lg bg-muted p-4 text-xs'}>{stack}</pre>}
    </main>
  );
}
