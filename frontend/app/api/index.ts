export const BASE_URL = typeof window === 'undefined' ? '' : (localStorage.getItem('api_url') ?? '');
