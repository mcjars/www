import axios from 'axios';
import { BASE_URL } from '@/api/index.ts';

export default async function apiDeleteUserOrganization(organization: number): Promise<void> {
  await axios.delete(`${BASE_URL}/api/user/organizations/${organization}`, {
    withCredentials: true,
  });
}
