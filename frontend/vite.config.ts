import path from 'node:path';

import { reactRouter } from '@react-router/dev/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [tailwindcss(), reactRouter()],
  resolve: {
    alias: {
      '~': path.resolve(import.meta.dirname, './app'),
    },
  },
  server: {
    host: true,
    port: 9000,
    allowedHosts: true,
    proxy: {
      '/api': {
        target: 'http://localhost:9000',
        ws: true,
      },
    },
  },
});
