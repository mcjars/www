import axios from 'axios';
import { BASE_URL } from '~/api/index.ts';

export default async function apiPostUserLogout(): Promise<void> {
  await axios.post(
    `${BASE_URL}/api/user/logout`,
    {},
    {
      withCredentials: true,
    },
  );
}
