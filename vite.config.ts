import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'node:path';

// Tauri expects a fixed port and dev server URL to match `tauri.conf.json#build.devUrl`
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  resolve: {
    alias: {
      $lib: resolve(__dirname, 'src/lib')
    }
  },
  // Multi-window: each window gets its own HTML entry
  build: {
    target: 'esnext',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        sticky: resolve(__dirname, 'sticky.html'),
        todo: resolve(__dirname, 'todo.html')
      }
    },
    // Tauri uses Chromium ≥89 / WebView2 (Edge); skip legacy polyfills for smaller bundles
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*']
});
