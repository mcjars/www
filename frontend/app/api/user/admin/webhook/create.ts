import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export type WebhookData = {
  organization: number | null;
  label: string | null;
  url: string;
  types: string[];
  enabled: boolean;
};

export default async function apiCreateUserAdminWebhook(data: WebhookData): Promise<number> {
  const { data: response } = await axios.post<{ id: number }>(`${BASE_URL}/api/user/admin/webhooks`, data, {
    withCredentials: true,
  });

  return response.id;
}
