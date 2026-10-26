/** Wire names from saved docs and @quiverai/sdk 0.9.4; see Phase 4 findings. */
export const BASE_URL = 'https://api.quiver.ai/v1';
export const GENERATIONS = '/svgs/generations';
export const ANIMATIONS = '/svgs/animations';
export type GenerateRequest = {
  model: string; prompt: string; instructions?: string; n?: number; stream?: false;
  references?: (string | {url: string} | {base64: string})[];
  temperature?: number; top_p?: number; presence_penalty?: number; max_output_tokens?: number;
};
export type AnimateRequest = {
  model: string; svg_source: {url: string} | {base64: string}; prompt?: string;
  reasoning_effort?: 'low' | 'medium' | 'high' | 'xhigh'; stream?: false;
  temperature?: number; max_output_tokens?: number;
};
export type SvgDocument = {svg: string; mime_type: 'image/svg+xml'};
export type Usage = {input_tokens: number; output_tokens: number; total_tokens: number};
export type SvgResponse = {created: number; id: string; credits?: number; data: SvgDocument[]; usage?: Usage};
export type AnimationResponse = Omit<SvgResponse, 'data' | 'usage'> & {
  data: (SvgDocument & {loop_period_ms?: number | null; opening_animation_ms?: number | null})[];
  svg_score?: number | null; usage?: Usage | null;
};
// Preserve catalog fields without guessing billing subtypes; consumers can narrow unknown fields.
export type ModelsResponse = {object: 'list'; data: ({id: string; [field: string]: unknown})[]};
export type ApiResult<T> = {body: T; request_id: string | null; environment: string | null};
export class QuiverError extends Error {
  constructor(public status: number, public code: string, message: string, public request_id: string | null) {
    super(message); this.name = 'QuiverError';
  }
}
export type ClientOptions = {
  fetch?: typeof fetch; getApiKey?: () => string | undefined;
  sleep?: (ms: number) => Promise<void>; now?: () => number;
  maxRetries?: number; maxRetryDelayMs?: number;
};
export class QuiverClient {
  private options: Required<ClientOptions>;
  constructor(options: ClientOptions = {}) {
    this.options = {
      fetch: (...args) => globalThis.fetch(...args),
      getApiKey: () => process.env.QUIVERAI_API_KEY,
      sleep: ms => new Promise(resolve => setTimeout(resolve, ms)), now: Date.now,
      maxRetries: 2, maxRetryDelayMs: 60_000, ...options,
    };
    if (!Number.isInteger(this.options.maxRetries) || this.options.maxRetries < 0 || this.options.maxRetries > 10 ||
        !Number.isFinite(this.options.maxRetryDelayMs) || this.options.maxRetryDelayMs < 0 || this.options.maxRetryDelayMs > 2_147_483_647) {
      throw new Error('Invalid Quiver retry bounds');
    }
  }
  private async request<T>(endpoint: string, body?: unknown): Promise<ApiResult<T>> {
    const key = this.options.getApiKey(); // Lazy: never read during import, construction, cache hits, or dry runs.
    if (!key?.trim()) throw new Error('Missing QUIVERAI_API_KEY');
    const redact = (value: string) => value.split(key).join('[redacted]');
    for (let attempt = 0; ; attempt++) {
      let response: Response;
      try {
        response = await this.options.fetch(BASE_URL + endpoint, {
          method: body === undefined ? 'GET' : 'POST', redirect: 'error',
          headers: {Authorization: `Bearer ${key}`, ...(body === undefined ? {} : {'Content-Type': 'application/json'})},
          ...(body === undefined ? {} : {body: JSON.stringify(body)}),
        });
      } catch { throw new Error('Quiver transport failed'); }
      const requestId = response.headers.get('X-Request-ID');
      if ((response.status === 429 || response.status === 503) && attempt < this.options.maxRetries) {
        const header = response.headers.get('Retry-After');
        const seconds = header !== null && /^\d+(\.\d+)?$/.test(header.trim()) ? Number(header) : NaN;
        const delay = header === null ? 1000 * 2 ** attempt : Number.isFinite(seconds) ? seconds * 1000 : Date.parse(header) - this.options.now();
        // Never retry earlier than requested. Excessive/invalid server delays surface the error instead.
        if (Number.isFinite(delay) && delay <= this.options.maxRetryDelayMs) {
          await response.body?.cancel();
          await this.options.sleep(Math.max(0, delay));
          continue;
        }
      }
      const data = await response.json().catch(() => null);
      if (!response.ok) {
        throw new QuiverError(typeof data?.status === 'number' ? data.status : response.status,
          redact(typeof data?.code === 'string' ? data.code : 'http_error'),
          redact(typeof data?.message === 'string' ? data.message : `Quiver HTTP ${response.status}`),
          typeof data?.request_id === 'string' ? redact(data.request_id) : requestId && redact(requestId));
      }
      if (!data || typeof data !== 'object') throw new Error('Invalid Quiver JSON response');
      return {body: data as T, request_id: requestId, environment: response.headers.get('x-quiver-environment')};
    }
  }
  generateSvg(req: GenerateRequest) {
    if (req.stream !== undefined && req.stream !== false) throw new Error('Streaming is not supported');
    return this.request<SvgResponse>(GENERATIONS, req);
  }
  animateSvg(req: AnimateRequest) {
    if (req.stream !== undefined && req.stream !== false) throw new Error('Streaming is not supported');
    return this.request<AnimationResponse>(ANIMATIONS, req);
  }
  listModels() { return this.request<ModelsResponse>('/models'); }
}
const client = new QuiverClient();
export const generateSvg = (req: GenerateRequest) => client.generateSvg(req);
export const animateSvg = (req: AnimateRequest) => client.animateSvg(req);
export const listModels = () => client.listModels();
