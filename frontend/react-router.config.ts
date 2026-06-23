import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import type { Config } from '@react-router/dev/config';

const STATIC_ROUTES = ['/', '/configs', '/lookup'];
const ROUTE_IDENTIFIER_OVERRIDES: Record<string, string> = {
  LEGACY_FABRIC: 'LEGACYFABRIC',
  LOOHP_LIMBO: 'LOOHPLIMBO',
};

function getTypeIdentifiers(): string[] {
  try {
    const schemaPath = fileURLToPath(new URL('../database/src/schema.ts', import.meta.url));
    const source = readFileSync(schemaPath, 'utf8');

    const block = source.match(/export const types = \[([\s\S]*?)\]/);
    if (!block) return [];

    return [...block[1].matchAll(/'([^']+)'/g)].map((match) => {
      const identifier = match[1];
      return ROUTE_IDENTIFIER_OVERRIDES[identifier] ?? identifier;
    });
  } catch {
    return [];
  }
}

export default {
  buildDirectory: 'lib',
  ssr: false,
  future: {
    v8_middleware: true,
    v8_splitRouteModules: true,
    v8_viteEnvironmentApi: true,
    v8_passThroughRequests: true,
    v8_trailingSlashAwareDataRequests: true,
  },
  prerender() {
    const typeRoutes = getTypeIdentifiers().flatMap((type) => [
      `/${type}/config`,
      `/${type}/versions`,
      `/${type}/statistics`,
    ]);

    return [...STATIC_ROUTES, ...typeRoutes];
  },
} satisfies Config;
