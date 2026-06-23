import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiAddUserOrganizationSubuser(organization: number, login: string): Promise<void> {
  await axios.post(
    `${BASE_URL}/api/user/organizations/${organization}/subusers`,
    {
      login: login.replace('@', ''),
    },
    {
      withCredentials: true,
    },
  );
}
