import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export type PatchOrganizationData = {
  name?: string;
  types?: string[];
  owner?: string;
  public?: boolean;
};

export default async function apiPatchUserOrganization(
  organization: number,
  data: PatchOrganizationData,
): Promise<void> {
  await axios.patch(`${BASE_URL}/api/user/organizations/${organization}`, data, {
    withCredentials: true,
  });
}
