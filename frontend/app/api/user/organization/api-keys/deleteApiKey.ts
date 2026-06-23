import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiDeleteUserOrganizationApiKey(organization: number, apiKey: number): Promise<void> {
  await axios.delete(`${BASE_URL}/api/user/organizations/${organization}/api-keys/${apiKey}`, {
    withCredentials: true,
  });
}
