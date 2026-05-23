import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';

function normalizeBasePath(value) {
  const trimmed = (value ?? '').trim();
  if (!trimmed || trimmed === '/') return undefined;
  const prefixed = trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
  return prefixed.replace(/\/+$/, '');
}

export default defineConfig({
  output: 'static',
  base: normalizeBasePath(process.env.DAILY_GAME_BASE_PATH ?? process.env.ASTRO_BASE_PATH),
  build: {
    inlineStylesheets: 'never',
  },
  integrations: [svelte()],
});
