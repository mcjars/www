import axios from 'axios';
import { BASE_URL } from '@/api/index.ts';

export default async function apiPostUserIniteDecline(organizationId: number): Promise<void> {
  await axios.post(
    `${BASE_URL}/api/user/invites/${organizationId}/decline`,
    {},
    {
      withCredentials: true,
    },
  );
}
