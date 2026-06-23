import { useQuery, useQueryClient } from '@tanstack/react-query';
import apiGetUserInfos, { User } from '~/api/user/infos.ts';

type UserMutator = (
  data?: User | null | ((current: User | null | undefined) => User | null | undefined),
  revalidate?: boolean,
) => void;

export function useAuth(): [User | null, UserMutator, boolean] {
  const queryClient = useQueryClient();
  const { data, isLoading } = useQuery({ queryKey: ['user'], queryFn: () => apiGetUserInfos() });

  const mutate: UserMutator = (next, revalidate = true) => {
    if (next !== undefined) {
      queryClient.setQueryData<User | null>(
        ['user'],
        (current) =>
          (typeof next === 'function'
            ? (next as (c: User | null | undefined) => User | null | undefined)(current)
            : next) ?? null,
      );
    }
    if (revalidate) void queryClient.invalidateQueries({ queryKey: ['user'] });
  };

  return [data ?? null, mutate, isLoading];
}
