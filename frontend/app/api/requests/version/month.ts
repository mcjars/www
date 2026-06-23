import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiGetVersionRequestsMonth(version: string, year: number, month: number) {
  const { data } = await axios.get<{
    requests: Record<
      string,
      {
        day: number;
        total: number;
        uniqueIps: number;
      }[]
    >;
  }>(`${BASE_URL}/api/v2/requests/version/${version}/history/${year}/${month}`);

  return {
    type: 'month',
    requests: data.requests,
  } as const;
}
