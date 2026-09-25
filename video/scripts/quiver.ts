import {parseArgs} from 'node:util';
import {readFile} from 'node:fs/promises';
import {QuiverClient, BASE_URL, GENERATIONS, ANIMATIONS, QuiverError, type GenerateRequest, type AnimateRequest} from '../src/quiver/client';
import {QuiverCache} from '../src/quiver/cache';
export async function main(argv: string[], deps: {client?: QuiverClient; cache?: QuiverCache; print?: (s: string) => void} = {}): Promise<number> {
  const print = deps.print ?? console.log;
  const [command, ...args] = argv;
  if (!['generate', 'animate', 'models'].includes(command)) throw new Error('Expected generate, animate, or models');
  const {values, positionals} = parseArgs({args, allowPositionals: true, options: {
    'dry-run': {type: 'boolean'}, ...(command === 'generate' ? {model: {type: 'string' as const}, instructions: {type: 'string' as const}, n: {type: 'string' as const}, reference: {type: 'string' as const, multiple: true}} : {}),
    ...(command === 'animate' ? {prompt: {type: 'string' as const}} : {}),
  }});
  if (positionals.length !== (command === 'models' ? 0 : 1)) throw new Error('Expected exactly one prompt or SVG path (none for models)');
  let body: GenerateRequest | AnimateRequest | undefined;
  let endpoint = '/models';
  if (command === 'generate') {
    const n = values.n === undefined ? 1 : Number(values.n);
    if (!Number.isInteger(n) || n < 1 || n > 16) throw new Error('--n must be an integer from 1 to 16');
    const referencePaths = (values.reference as string[] | undefined) ?? [];
    if (referencePaths.length > 14) throw new Error('--reference accepts at most 14 images');
    const references = await Promise.all(referencePaths.map(async path => ({base64: (await readFile(path)).toString('base64')})));
    body = {model: typeof values.model === 'string' ? values.model : 'arrow-2', prompt: positionals[0], ...(typeof values.instructions !== 'string' ? {} : {instructions: values.instructions}), ...(references.length ? {references} : {}), n};
    endpoint = GENERATIONS;
  } else if (command === 'animate') {
    body = {model: 'arrow-2', svg_source: {base64: (await readFile(positionals[0])).toString('base64')}, ...(typeof values.prompt !== 'string' ? {} : {prompt: values.prompt})};
    endpoint = ANIMATIONS;
  }
  if (values['dry-run']) {
    print(JSON.stringify({method: body === undefined ? 'GET' : 'POST', url: BASE_URL + endpoint, ...(body === undefined ? {} : {body})}, null, 2));
    return 0;
  }
  const client = deps.client ?? new QuiverClient();
  if (!body) print(JSON.stringify((await client.listModels()).body, null, 2));
  else {
    const request = body;
    const entries = await (deps.cache ?? new QuiverCache()).getOrCreate(endpoint, request,
      () => endpoint === GENERATIONS ? client.generateSvg(request as GenerateRequest) : client.animateSvg(request as AnimateRequest));
    for (const entry of entries) print(`assets/quiver/${entry.hash}.svg`);
  }
  return 0;
}
if (require.main === module) main(process.argv.slice(2)).then(code => {process.exitCode = code;}).catch(error => {
  console.error(error instanceof QuiverError ? `${error.status} ${error.code}: ${error.message} (request_id=${error.request_id ?? 'unavailable'})` : error.message);
  process.exitCode = 1;
});
