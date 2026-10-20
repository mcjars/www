import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiDeleteUserAdminWebhook(webhook: number): Promise<void> {
  await axios.delete(`${BASE_URL}/api/user/admin/webhooks/${webhook}`, {
    withCredentials: true,
  });
}
