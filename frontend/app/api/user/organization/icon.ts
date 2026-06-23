import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiPostUserOrganizationIcon(organization: number, icon: File): Promise<string> {
  const { data } = await axios.post<{ url: string }>(`${BASE_URL}/api/user/organizations/${organization}/icon`, icon, {
    withCredentials: true,
  });

  return data.url;
}
