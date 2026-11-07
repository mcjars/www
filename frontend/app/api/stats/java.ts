import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiGetJavaStats() {
  const { data } = await axios.get<{
    java: Record<string, Record<string, number>>;
  }>(`${BASE_URL}/api/v2/stats/java`);

  return data.java;
}
