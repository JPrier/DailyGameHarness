import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';

export default defineConfig({
  output: 'static',
  build: {
    inlineStylesheets: 'never',
  },
  integrations: [svelte()],
});
