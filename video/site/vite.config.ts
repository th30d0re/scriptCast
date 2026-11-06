import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import fs from 'node:fs';
const video = path.resolve(__dirname, '..');
const planPath = process.env.STAGE_PLAN ?? path.join(video, 'episodes/_demo/stage_plan.json');
export default defineConfig({
  root: __dirname, publicDir: path.join(video,'public'),
  plugins: [react(), {
    name: 'stage-default-plan',
    configureServer(server) {
      server.middlewares.use('/stage/default.json', (_req,res) => {
        try {res.setHeader('Content-Type','application/json'); res.end(fs.readFileSync(planPath));}
        catch {res.statusCode=404; res.end('Compile a Stage plan first.');}
      });
    },
    generateBundle() {this.emitFile({type:'asset',fileName:'stage/default.json',source:fs.readFileSync(planPath)});}
  }],
  resolve: {dedupe:['react','react-dom','remotion','d3-geo'], alias: {
    '@episode': process.env.STAGE_EPISODE_DIR ?? path.join(video,'episodes/_demo'), '@stage':path.join(video,'src/stage/api.ts'),
    react:path.join(video,'node_modules/react'), 'react-dom':path.join(video,'node_modules/react-dom'), remotion:path.join(video,'node_modules/remotion'), 'd3-geo':path.join(video,'node_modules/d3-geo')
  }},
  server: {fs: {allow:[video, ...(process.env.STAGE_EPISODE_DIR ? [process.env.STAGE_EPISODE_DIR] : [])]}},
  build: {outDir:path.join(__dirname,'dist'), emptyOutDir:true}
});
