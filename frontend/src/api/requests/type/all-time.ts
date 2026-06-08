import axios from 'axios';
import { BASE_URL } from '@/api/index.ts';

export default async function apiGetTypeRequestsAllTime(type: string) {
  const { data } = await axios.get<{
    requests: {
      root: {
        total: number;
        uniqueIps: number;
      };

      versions: Record<
        string,
        {
          total: number;
          uniqueIps: number;
        }
      >;
    };
  }>(`${BASE_URL}/api/v2/requests/${type.toUpperCase()}`);

  return data.requests;
}
