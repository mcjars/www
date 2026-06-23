import axios from 'axios';
import { BASE_URL } from '@/api/index.ts';

export default async function apiGetVersionRequestsAllTime(version: string) {
  const { data } = await axios.get<{
    requests: Record<
      string,
      {
        total: number;
        uniqueIps: number;
      }
    >;
  }>(`${BASE_URL}/api/v2/requests/version/${version}`);

  return {
    type: 'all-time',
    requests: data.requests,
  } as const;
}
