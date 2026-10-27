import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 5173,
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:42128',
        changeOrigin: true,
      },
    },
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: {
          'three-vrm': ['three', '@pixiv/three-vrm'],
          'vendor-ui': ['solid-js', 'lucide-solid'],
          'vendor-markdown': ['marked', 'prismjs', 'diff'],
        },
      },
    },
  },
});
