import useSWR, { KeyedMutator } from 'swr';
import apiGetUserInfos, { User } from '@/api/user/infos.ts';

export function useAuth(): [User | null, KeyedMutator<User | null>, boolean] {
  const { data, mutate, isLoading } = useSWR('user', () => apiGetUserInfos(), { revalidateOnFocus: false });

  return [data || null, mutate, isLoading];
}
