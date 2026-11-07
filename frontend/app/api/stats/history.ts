import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiGetBuildsHistory(year: number, month: number) {
  const { data } = await axios.get<{
    builds: Record<
      string,
      {
        day: number;
        builds: number;
      }[]
    >;
  }>(`${BASE_URL}/api/v2/stats/history/${year}/${month}`);

  return data.builds;
}
