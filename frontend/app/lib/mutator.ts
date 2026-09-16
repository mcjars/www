import { QueryClient } from '@tanstack/react-query';

export type Mutator<T> = (
  data?: T | null | ((current: T | undefined) => T | null | undefined),
  revalidate?: boolean,
) => void;

export function createMutator<T>(queryClient: QueryClient, queryKey: unknown[]): Mutator<T> {
  return (data, revalidate = true) => {
    if (data !== undefined) {
      queryClient.setQueryData<T>(
        queryKey,
        (current) =>
          (typeof data === 'function'
            ? (data as (c: T | undefined) => T | null | undefined)(current)
            : data) as T | undefined,
      );
    }
    if (revalidate) void queryClient.invalidateQueries({ queryKey });
  };
}
