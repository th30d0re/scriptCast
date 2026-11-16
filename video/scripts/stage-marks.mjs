#!/usr/bin/env node
/*
 * Extract review marks from an exact Stage frame.
 *
 * Remotion 4.0.354's renderer owns its normal render page, so this starts a
 * tiny static server for the bundle and opens the StageMarks composition in
 * the same Chrome supplied by @remotion/renderer. StageMarks publishes the
 * result of collectMarks() on window.__scriptcastStageMarks.
 */
import {bundle} from '@remotion/bundler';
import {openBrowser} from '@remotion/renderer';
// Internal APIs intentionally pinned to the workspace's Remotion 4.0.354.
import {createRequire} from 'node:module';
import {createReadStream, existsSync, realpathSync, statSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {createServer} from 'node:http';
import {dirname, extname, join, normalize, resolve, sep} from 'node:path';

// Resolve via the public entry point: package exports hide the dist subpaths.
const require = createRequire(import.meta.url);
const rendererDist = dirname(require.resolve('@remotion/renderer'));
const {setPropsAndEnv} = require(join(rendererDist, 'set-props-and-env.js'));
const {seekToFrame, waitForReady} = require(join(rendererDist, 'seek-to-frame.js'));
const {puppeteerEvaluateWithCatchAndTimeout} = require(join(rendererDist, 'puppeteer-evaluate.js'));

// The programmatic bundler does not load remotion.config.ts. Keep resolution
// identical to its override, including dependencies hoisted by npm workspaces.
const video = resolve(process.env.STAGE_VIDEO_DIR ?? fileURLToPath(new URL('..', import.meta.url)));
const roots = [join(video, 'node_modules'), join(video, '..', 'node_modules')];
const dep = (name) => realpathSync(join(roots.find(root => existsSync(join(root, name))) ?? roots[0], name));
const webpackOverride = (config) => ({
  ...config,
  resolve: {
    ...config.resolve,
    symlinks: true,
    alias: {
      ...config.resolve?.alias,
      '@episode': process.env.STAGE_EPISODE_DIR ?? join(video, 'episodes/_demo'),
      '@stage': join(video, 'src/stage/api.ts'),
      react: dep('react'),
      'react-dom': dep('react-dom'),
      remotion: dep('remotion'),
      'd3-geo': dep('d3-geo'),
    },
    modules: [...roots, ...(config.resolve?.modules ?? ['node_modules'])],
  },
});

const args = process.argv.slice(2);
const take = (name) => {
  const index = args.indexOf(name);
  if (index < 0 || !args[index + 1]) throw new Error(`Missing ${name}`);
  return args[index + 1];
};
const planPath = resolve(take('--plan'));
const frame = Number(take('--frame'));
const output = args.includes('--output') ? resolve(take('--output')) : null;
if (!Number.isInteger(frame) || frame < 0) throw new Error('--frame must be a non-negative integer');

const mime = {'.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.map': 'application/json', '.woff': 'font/woff', '.woff2': 'font/woff2', '.svg': 'image/svg+xml', '.png': 'image/png'};
const serve = async (directory) => {
  const root = resolve(directory);
  const server = createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url ?? '/', 'http://localhost').pathname);
    const filename = resolve(root, normalize(pathname).replace(/^[/\\]+/, '') || 'index.html');
    if (filename !== root && !filename.startsWith(root + sep)) { response.writeHead(403); response.end(); return; }
    const file = existsSync(filename) && statSync(filename).isFile() ? filename : join(root, 'index.html');
    if (!existsSync(file)) { response.writeHead(404); response.end(); return; }
    response.writeHead(200, {'content-type': mime[extname(file)] ?? 'application/octet-stream'});
    createReadStream(file).pipe(response);
  });
  await new Promise((done, fail) => { server.once('error', fail); server.listen(0, '127.0.0.1', done); });
  return server;
};

let browser;
let server;
const diagnostics = [];
const capture = (message) => {
  diagnostics.push(message);
  if (diagnostics.length > 200) diagnostics.shift();
};
try {
  const bundleLocation = await bundle({entryPoint: join(video, 'src/stage/index.ts'), webpackOverride});
  server = await serve(bundleLocation);
  const address = server.address();
  const props = JSON.stringify(JSON.parse(await (await import('node:fs/promises')).readFile(planPath, 'utf8')));
  browser = await openBrowser('chrome', {
    browserExecutable: process.env.SCRIPTCAST_CHROME ?? null,
    chromiumOptions: {},
    logLevel: 'info',
  });
  // Remotion 4.0.354 returns its own HeadlessBrowser, not Puppeteer.
  const page = await browser.newPage({
    context: null, logLevel: 'info', indent: false, pageIndex: 0,
    onBrowserLog: log => capture(`[${log.type}] ${log.text}\n${JSON.stringify(log.stackTrace)}`),
  });
  page.on('error', error => capture(`[page error] ${error.stack ?? error.message}`));
  const timeoutInMilliseconds = 120000;
  await setPropsAndEnv({
    serializedInputPropsWithCustomSchema: props, envVariables: {}, page,
    serveUrl: `http://127.0.0.1:${address.port}/index.html`, initialFrame: frame,
    timeoutInMilliseconds, proxyPort: address.port, retriesRemaining: 2,
    audioEnabled: false, videoEnabled: true, indent: false, logLevel: 'info',
    onServeUrlVisited: () => {}, isMainTab: true,
    mediaCacheSizeInBytes: null, initialMemoryAvailable: null,
  });
  const evaluate = async (pageFunction, args = []) => {
    const {value} = await puppeteerEvaluateWithCatchAndTimeout({
      page, pageFunction, args, frame, timeoutInMilliseconds,
    });
    return value;
  };
  // selectComposition/getCompositions mount the evaluation root first. Merely
  // loading index.html defines the API but does not install CompositionManager.
  await evaluate(() => window.remotion_setBundleMode({type: 'evaluation'}));
  await waitForReady({page, timeoutInMilliseconds, frame: null, indent: false, logLevel: 'info'});
  const composition = await evaluate(() => window.remotion_calculateComposition('StageMarks'));
  if (frame >= composition.durationInFrames) throw new Error(`Frame ${frame} is outside StageMarks duration`);
  await page.setViewport({width: composition.width, height: composition.height, deviceScaleFactor: 1});
  await evaluate(c => window.remotion_setBundleMode({
    type: 'composition', compositionName: c.id,
    serializedResolvedPropsWithSchema: c.serializedResolvedPropsWithCustomSchema,
    compositionDurationInFrames: c.durationInFrames, compositionFps: c.fps,
    compositionHeight: c.height, compositionWidth: c.width,
    compositionDefaultCodec: c.defaultCodec, compositionDefaultOutName: c.defaultOutName,
    compositionDefaultVideoImageFormat: c.defaultVideoImageFormat,
    compositionDefaultPixelFormat: c.defaultPixelFormat,
  }), [composition]);
  await seekToFrame({frame, page, composition: composition.id, timeoutInMilliseconds,
    logLevel: 'info', indent: false, attempt: 0});
  // Recollect after delayRender and fonts have settled, not at the first layout.
  const marks = await evaluate(() => window.__scriptcastCollectStageMarks?.());
  if (!marks || marks.frame !== frame) throw new Error(`StageMarks did not publish frame ${frame}`);
  const result = {frame, marks: marks?.marks ?? []};
  const text = JSON.stringify(result, null, 2) + '\n';
  if (output) await (await import('node:fs/promises')).writeFile(output, text);
  else process.stdout.write(text);
} catch (error) {
  console.error(`StageMarks failed: ${error.stack ?? error}`);
  console.error('Captured browser diagnostics (last 200):\n' + (diagnostics.join('\n') || '(none)'));
  process.exitCode = 1;
} finally {
  try {
    if (browser) await browser.close({silent: true});
  } catch (error) {
    console.error(`StageMarks browser cleanup failed: ${error.stack ?? error}`);
    process.exitCode = 1;
  } finally {
    if (server) await new Promise(done => server.close(done));
  }
}
