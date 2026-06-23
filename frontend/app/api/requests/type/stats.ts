import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiGetTypeStats(type: string) {
  const { data } = await axios.get<{
    stats: {
      size: {
        total: {
          jar: number;
          zip: number;
        };
      };
    };
  }>(`${BASE_URL}/api/v2/stats/${type.toUpperCase()}`);

  return data.stats;
}
