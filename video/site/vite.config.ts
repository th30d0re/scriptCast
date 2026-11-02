import {defineConfig, type Plugin} from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import fs from 'node:fs';
import {parseNotes} from '../src/stage/review/schema';
const video = path.resolve(__dirname, '..');
const planPath = process.env.STAGE_PLAN ?? path.join(video, 'episodes/_demo/stage_plan.json');
const notesPath = path.join(video, '..', 'review', 'notes.json');

function reviewNotes(): Plugin {
  const read = () => fs.existsSync(notesPath) ? fs.readFileSync(notesPath) : Buffer.from('[]\n');
  return {
    name: 'stage-review-notes',
    configureServer(server) {
      server.middlewares.use('/review/notes.json', (req, res) => {
        res.setHeader('Cache-Control', 'no-store');
        if (req.method === 'GET' || req.method === 'HEAD') {
          res.setHeader('Content-Type', 'application/json');
          res.end(req.method === 'HEAD' ? undefined : read());
          return;
        }
        if (req.method !== 'PUT') {res.statusCode = 405; res.end('Use GET or PUT'); return;}
        const chunks: Buffer[] = [];
        req.on('data', chunk => chunks.push(Buffer.from(chunk)));
        req.on('end', () => {
          try {
            const text = Buffer.concat(chunks).toString('utf8');
            if (text.length > 1_000_000) throw new Error('notes payload is too large');
            const notes = parseNotes(JSON.parse(text));
            if (!notes) throw new Error('notes must be a list of {scene, element, ms, cue, note, author, status:"open"}');
            fs.mkdirSync(path.dirname(notesPath), {recursive: true});
            fs.writeFileSync(notesPath, JSON.stringify(notes, null, 2) + '\n');
            res.setHeader('Content-Type', 'application/json');
            res.end('{"ok":true}\n');
          } catch (error) {
            res.statusCode = 400;
            res.end(error instanceof Error ? error.message : String(error));
          }
        });
      });
    },
  };
}

export default defineConfig({
  root: __dirname, publicDir: path.join(video,'public'),
  plugins: [react(), reviewNotes(), {
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
  server: {fs: {allow:[video, ...[path.resolve(video, '../../scriptCast-stage/video/node_modules')].filter(dir => fs.existsSync(dir)), ...(process.env.STAGE_EPISODE_DIR ? [process.env.STAGE_EPISODE_DIR] : [])]}},
  build: {outDir:path.join(__dirname,'dist'), emptyOutDir:true}
});
