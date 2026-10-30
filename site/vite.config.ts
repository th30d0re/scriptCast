import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';

// Card assets (assets/*.svg) are shared with the video, so serve video/public as is.
export default defineConfig({
  plugins: [react()],
  publicDir: '../video/public',
});
