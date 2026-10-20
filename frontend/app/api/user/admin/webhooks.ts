import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';
import { Paginated } from '~/api/user/admin/types.ts';

export type Webhook = {
  id: number;
  organization: {
    id: number;
    name: string;
    verified: boolean;
  } | null;

  label: string | null;
  url: string;
  types: string[];
  enabled: boolean;

  successful: number;
  failed: number;
};

export default async function apiGetUserAdminWebhooks(
  page: number,
  perPage: number,
  search?: string,
): Promise<Paginated<Webhook>> {
  const { data } = await axios.get<{ webhooks: Paginated<Webhook> }>(`${BASE_URL}/api/user/admin/webhooks`, {
    params: { page, per_page: perPage, search: search || undefined },
    withCredentials: true,
  });

  return data.webhooks;
}
