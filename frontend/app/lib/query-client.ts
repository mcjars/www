import { QueryClient } from '@tanstack/react-query';

let client: QueryClient | undefined;

function createClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
        staleTime: Number.POSITIVE_INFINITY,
      },
    },
  });
}

export function getQueryClient() {
  if (typeof window === 'undefined') return createClient();
  if (!client) client = createClient();
  return client;
}
