import axios from 'axios';
import { BASE_URL } from '@/api/index.ts';

export default async function apiPostUserOrganizationUpdateBuildData(organization: number): Promise<void> {
  await axios.post(`${BASE_URL}/api/user/organizations/${organization}/update-build-data`, undefined, {
    withCredentials: true,
  });
}
