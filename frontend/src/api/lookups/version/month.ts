import axios from 'axios';
import { BASE_URL } from '@/api/index.ts';

export default async function apiGetTypeVersionLookupsMonth(type: string, year: number, month: number) {
  const { data } = await axios.get<{
    versions: Record<
      string,
      {
        day: number;
        total: number;
        uniqueIps: number;
      }[]
    >;
  }>(`${BASE_URL}/api/v2/lookups/versions/${type.toUpperCase()}/history/${year}/${month}`);

  return data.versions;
}
