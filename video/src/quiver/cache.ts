import {createHash, randomUUID} from 'node:crypto';
import {mkdir, readFile, writeFile, rename} from 'node:fs/promises';
import {resolve, join} from 'node:path';
import type {GenerateRequest, AnimateRequest, ApiResult, SvgResponse, AnimationResponse} from './client';
export type Request = GenerateRequest | AnimateRequest;
export type Entry = {prompt: string | null; model: string; request_id: string | null; date: string; endpoint: string; hash: string; request_hash: string; output_index: number; output_count: number};
export function canonicalJson(value: unknown): string {
  const sort = (v: unknown): unknown => Array.isArray(v) ? v.map(sort) : v && typeof v === 'object'
    ? Object.fromEntries(Object.entries(v).filter(([, x]) => x !== undefined).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).map(([k, x]) => [k, sort(x)])) : v;
  return JSON.stringify(sort(value));
}
// Authentication is separate from the request type, and is never passed to the hash or manifest.
export const cacheKey = (endpoint: string, request: Request, outputIndex = 0): string =>
  createHash('sha256').update(canonicalJson({endpoint, request, output_index: outputIndex})).digest('hex');
export class QuiverCache {
  constructor(public directory = resolve('public/assets/quiver')) {}
  private async manifest(): Promise<Entry[]> {
    try {
      const entries = JSON.parse(await readFile(join(this.directory, 'manifest.json'), 'utf8'));
      if (!Array.isArray(entries)) throw new Error('Invalid Quiver manifest');
      return entries;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') return [];
      throw error;
    }
  }
  async getOrCreate(endpoint: string, request: Request, produce: () => Promise<ApiResult<SvgResponse | AnimationResponse>>): Promise<Entry[]> {
    const hash = cacheKey(endpoint, request);
    const manifest = await this.manifest();
    const hits = manifest.filter(e => e.request_hash === hash).sort((a, b) => a.output_index - b.output_index);
    if (hits.length && hits.length === hits[0].output_count && hits.every((e, i) => e.output_index === i && e.hash === cacheKey(endpoint, request, i))) {
      try {
        await Promise.all(hits.map(e => readFile(join(this.directory, `${e.hash}.svg`))));
        return hits;
      } catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    }
    const result = await produce();
    const docs = result.body.data;
    if (!Array.isArray(docs) || !docs.length || docs.some(d => typeof d.svg !== 'string' || d.mime_type !== 'image/svg+xml')) throw new Error('Invalid Quiver SVG response');
    if (result.environment === 'test' || docs.some(d => /data-quiverai-sandbox\s*=/.test(d.svg))) throw new Error('Sandbox results cannot enter the production asset cache');
    await mkdir(this.directory, {recursive: true});
    const entries: Entry[] = [];
    for (const [i, doc] of docs.entries()) {
      const entry: Entry = {prompt: request.prompt ?? null, model: request.model, request_id: result.request_id,
        date: new Date().toISOString(), endpoint, hash: cacheKey(endpoint, request, i), request_hash: hash, output_index: i, output_count: docs.length};
      await writeFile(join(this.directory, `${entry.hash}.svg`), doc.svg);
      entries.push(entry);
    }
    // Atomic manifest replacement, re-read to preserve completed unrelated requests.
    const latest = await this.manifest();
    const temp = join(this.directory, `manifest.${randomUUID()}.tmp`);
    await writeFile(temp, JSON.stringify([...latest.filter(e => e.request_hash !== hash), ...entries], null, 2) + '\n');
    await rename(temp, join(this.directory, 'manifest.json'));
    return entries;
  }
}
