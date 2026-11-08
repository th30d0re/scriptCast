import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createKitReviewMiddleware} from './kit-review-middleware.mjs';

// Card assets (assets/*.svg) are shared with the video, so serve video/public as is.
export default defineConfig({
  plugins: [react(), {
    name: 'kit-review',
    configureServer(server) {
      server.middlewares.use(createKitReviewMiddleware(path.resolve(fileURLToPath(new URL('.', import.meta.url)), '../review/kit-review.json')));
    },
  }],
  resolve: {dedupe: ['remotion', '@remotion/player', 'react', 'react-dom'], alias: {'@stage': path.resolve(fileURLToPath(new URL('.', import.meta.url)), '../video/src/stage'), '@video': path.resolve(fileURLToPath(new URL('.', import.meta.url)), '../video/src')}},
  publicDir: '../video/public',
});
