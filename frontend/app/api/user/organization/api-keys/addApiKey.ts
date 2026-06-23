import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiAddUserOrganizationApiKey(organization: number, name: string): Promise<string> {
  const { data } = await axios.post<{
    key: string;
  }>(
    `${BASE_URL}/api/user/organizations/${organization}/api-keys`,
    {
      name,
    },
    {
      withCredentials: true,
    },
  );

  return data.key;
}
