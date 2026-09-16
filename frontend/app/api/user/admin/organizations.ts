import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';
import { Paginated } from '~/api/user/admin/types.ts';
import { Organization } from '~/api/user/organizations.ts';

export default async function apiGetUserAdminOrganizations(
  page: number,
  perPage: number,
  search?: string,
): Promise<Paginated<Organization>> {
  const { data } = await axios.get<{ organizations: Paginated<Organization> }>(
    `${BASE_URL}/api/user/admin/organizations`,
    {
      params: { page, per_page: perPage, search: search || undefined },
      withCredentials: true,
    },
  );

  return data.organizations;
}
