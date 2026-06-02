import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  root: 'frontend',
  plugins: [wasm()],
  server: {
    port: 8080,
  },
});
