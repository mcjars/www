import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiGetTypeRequestsMonth(type: string, year: number, month: number) {
  const { data } = await axios.get<{
    requests: {
      day: number;
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
    }[];
  }>(`${BASE_URL}/api/v2/requests/${type.toUpperCase()}/history/${year}/${month}`);

  return data.requests;
}
