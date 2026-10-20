import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';
import { Webhook } from '~/api/user/admin/webhooks.ts';
import { WebhookData } from '~/api/user/admin/webhook/create.ts';

export default async function apiPatchUserAdminWebhook(
  webhook: number,
  data: WebhookData & { resetCounters?: boolean },
): Promise<Webhook> {
  const { data: response } = await axios.patch<{ webhook: Webhook }>(
    `${BASE_URL}/api/user/admin/webhooks/${webhook}`,
    data,
    {
      withCredentials: true,
    },
  );

  return response.webhook;
}
