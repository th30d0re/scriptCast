import assert from 'node:assert/strict';
import {randomBytes} from 'node:crypto';
import {mkdtemp, readFile, writeFile, readdir, unlink, rmdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {QuiverClient, QuiverError, BASE_URL, GENERATIONS, ANIMATIONS, type ClientOptions} from '../src/quiver/client';
import {QuiverCache, cacheKey} from '../src/quiver/cache';
import {main} from './quiver';
// Synthetic credential exists only in memory; no environment access or real key.
const token = randomBytes(24).toString('hex');
const svg = '<svg xmlns="http://www.w3.org/2000/svg"><circle r="1"/></svg>';
const generation = {created: 1, id: 'synthetic-generation', data: [{svg, mime_type: 'image/svg+xml'}], usage: {input_tokens: 1, output_tokens: 2, total_tokens: 3}};
const animation = {...generation, id: 'synthetic-animation', data: [{svg, mime_type: 'image/svg+xml', loop_period_ms: 1000, opening_animation_ms: null}], svg_score: null, usage: null};
const json = (data: unknown, status = 200, headers: Record<string, string> = {}) => new Response(JSON.stringify(data), {status, headers: {'Content-Type': 'application/json', 'X-Request-ID': 'synthetic-request', ...headers}});
const req = {model: 'arrow-2', prompt: 'A circle', instructions: 'Flat', n: 1};
const calls: {url: string; init: RequestInit}[] = [];
const client = (options: ClientOptions = {}) => new QuiverClient({getApiKey: () => token, fetch: async (url, init) => {
  calls.push({url: String(url), init: init!});
  return json(String(url).endsWith(ANIMATIONS) ? animation : String(url).endsWith('/models') ? {object: 'list', data: []} : generation);
}, ...options});
async function check() {
  const c = client();
  const got = await c.generateSvg(req);
  assert.deepEqual(got.body, generation);
  assert.equal(got.request_id, 'synthetic-request');
  assert.equal(calls[0].url, BASE_URL + GENERATIONS);
  assert.equal(calls[0].init.method, 'POST');
  assert.equal(calls[0].init.body, JSON.stringify(req));
  assert.equal(calls[0].init.redirect, 'error');
  assert.deepEqual(calls[0].init.headers, {Authorization: `Bearer ${token}`, 'Content-Type': 'application/json'});
  const ar = {model: 'arrow-2', svg_source: {base64: Buffer.from(svg).toString('base64')}, prompt: 'Rotate'};
  assert.deepEqual((await c.animateSvg(ar)).body, animation);
  assert.equal(calls[1].url, BASE_URL + ANIMATIONS);
  assert.equal(calls[1].init.body, JSON.stringify(ar));
  assert.deepEqual(calls[1].init.headers, calls[0].init.headers);
  assert.deepEqual((await c.listModels()).body, {object: 'list', data: []});
  assert.equal(calls[2].url, BASE_URL + '/models');
  assert.equal(calls[2].init.method, 'GET');
  assert.equal(calls[2].init.body, undefined);
  assert.deepEqual(calls[2].init.headers, {Authorization: `Bearer ${token}`});
  await assert.rejects(client({getApiKey: () => undefined, fetch: async () => {throw new Error('must not fetch');}}).generateSvg(req), /^Error: Missing QUIVERAI_API_KEY$/);
  let reads = 0;
  const lazy = client({getApiKey: () => {reads++; return token;}});
  assert.equal(reads, 0);
  await lazy.listModels(); assert.equal(reads, 1);
  for (const status of [429, 503]) {
    let attempts = 0; const delays: number[] = [];
    const retry = client({fetch: async () => ++attempts === 1 ? json({status, code: 'busy', message: 'Wait', request_id: 'retry'}, status, {'Retry-After': '2'}) : json(generation), sleep: async ms => {delays.push(ms);}});
    await retry.generateSvg(req); assert.equal(attempts, 2); assert.deepEqual(delays, [2000]);
  }
  let attempts = 0; const delays: number[] = [];
  const limited = client({fetch: async () => {attempts++; return json({status: 503, code: 'model_unavailable', message: 'Wait', request_id: 'bounded'}, 503, {'Retry-After': 'Thu, 01 Jan 1970 00:00:03 GMT'});}, now: () => 1000, sleep: async ms => {delays.push(ms);}});
  await assert.rejects(limited.generateSvg(req), (error: unknown) => error instanceof QuiverError && error.request_id === 'bounded' && error.status === 503 && error.code === 'model_unavailable' && error.message === 'Wait');
  assert.equal(attempts, 3); assert.deepEqual(delays, [2000, 2000]);
  for (const delay of ['1000000', 'invalid']) {
    let count = 0;
    await assert.rejects(client({fetch: async () => {count++; return json({}, 429, {'Retry-After': delay});}, sleep: async () => {assert.fail('must not retry');}}).listModels(), QuiverError);
    assert.equal(count, 1);
  }
  await assert.rejects(client({fetch: async () => json({status: 400, code: 'invalid_request', message: `Rejected ${token}`, request_id: 'error-fixture'}, 400)}).generateSvg(req), (error: unknown) => error instanceof QuiverError && error.request_id === 'error-fixture' && !error.message.includes(token));
  console.log('PASS: documented generation/animation/models wire requests, lazy credentials, missing key, retries/date/bounds, typed errors and redaction.');
  const dir = await mkdtemp(join(tmpdir(), 'scriptcast-quiver-'));
  try {
    const cache = new QuiverCache(dir);
    const before = calls.length;
    const entries = await cache.getOrCreate(GENERATIONS, req, () => c.generateSvg(req));
    assert.equal(calls.length, before + 1);
    const reversed = {n: 1, instructions: 'Flat', prompt: 'A circle', model: 'arrow-2'};
    assert.equal(cacheKey(GENERATIONS, req), cacheKey(GENERATIONS, reversed));
    assert.notEqual(cacheKey(GENERATIONS, req), cacheKey(ANIMATIONS, req));
    assert.equal(cacheKey(ANIMATIONS, ar), cacheKey(ANIMATIONS, {prompt: 'Rotate', svg_source: {base64: ar.svg_source.base64}, model: 'arrow-2'}));
    const changedAuth = client({getApiKey: () => {throw new Error('cache hit must not read credentials');}, fetch: async () => {assert.fail('cache hit must not fetch');}});
    assert.deepEqual(await cache.getOrCreate(GENERATIONS, reversed, () => changedAuth.generateSvg(reversed)), entries);
    assert.equal(calls.length, before + 1);
    assert.equal(await readFile(join(dir, entries[0].hash + '.svg'), 'utf8'), svg);
    const manifest = await readFile(join(dir, 'manifest.json'), 'utf8');
    assert(!manifest.includes(token));
    assert.deepEqual(JSON.parse(manifest), entries);
    assert.equal(entries[0].prompt, req.prompt); assert.equal(entries[0].model, req.model);
    assert.equal(entries[0].endpoint, GENERATIONS); assert.equal(entries[0].request_id, 'synthetic-request');
    assert(Number.isFinite(Date.parse(entries[0].date)));
    const multi = {...req, n: 2};
    const batch = await cache.getOrCreate(GENERATIONS, multi, async () => ({...got, body: {...generation, data: [...generation.data, ...generation.data]} as typeof got.body}));
    assert.equal(batch.length, 2); assert.notEqual(batch[0].hash, batch[1].hash);
    assert.deepEqual(await cache.getOrCreate(GENERATIONS, multi, async () => {assert.fail('batch cache miss');}), batch);
    const animated = await cache.getOrCreate(ANIMATIONS, ar, () => c.animateSvg(ar));
    assert.equal(await readFile(join(dir, animated[0].hash + '.svg'), 'utf8'), svg);
    await assert.rejects(cache.getOrCreate(GENERATIONS, {...req, prompt: 'Sandbox'}, async () => ({...got, environment: 'test'})), /Sandbox/);
    const path = join(dir, 'input.svg'); await writeFile(path, svg);
    const noSideEffects = client({getApiKey: () => {assert.fail('dry run read credentials');}, fetch: async () => {assert.fail('dry run fetch');}});
    for (const [argv, expected] of [
      [['generate', req.prompt, '--instructions', 'Flat', '--dry-run'], {method: 'POST', url: BASE_URL + GENERATIONS, body: req}],
      [['animate', path, '--prompt', 'Rotate', '--dry-run'], {method: 'POST', url: BASE_URL + ANIMATIONS, body: ar}],
      [['models', '--dry-run'], {method: 'GET', url: BASE_URL + '/models'}],
    ] as const) {
      const lines: string[] = [];
      assert.equal(await main([...argv], {client: noSideEffects, print: s => lines.push(s)}), 0);
      assert.deepEqual(JSON.parse(lines.join('')), expected);
      assert(!lines.join('').includes(token)); assert(!lines.join('').includes('Authorization'));
    }
    await assert.rejects(main(['generate', 'Circle', '--n', '0', '--dry-run']), /--n/);
    console.log('PASS: canonical cache keys, credential-independent hits with zero fetches, SVG/manifest persistence, multi-output and animation caching, sandbox rejection, and exact key-free CLI dry runs.');
  } finally {
    for (const name of await readdir(dir)) await unlink(join(dir, name));
    await rmdir(dir);
  }
}
check().catch(() => {console.error('FAIL: offline Quiver checks (details suppressed to protect synthetic credentials)'); process.exitCode = 1;});
