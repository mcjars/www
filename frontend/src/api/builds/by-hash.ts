import axios from 'axios';
import { PartialMinecraftBuild } from '@/api/builds/index.ts';
import { BASE_URL } from '@/api/index.ts';

export default async function apiGetBuild(build: string): Promise<PartialMinecraftBuild> {
  const { data } = await axios.get<{
    build: PartialMinecraftBuild;
  }>(`${BASE_URL}/api/v3/builds/${build}/versions`);

  return data.build;
}
