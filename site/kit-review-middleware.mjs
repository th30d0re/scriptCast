import fs from 'node:fs';
import path from 'node:path';

const blank = () => ({version: 1, items: {}});
const priorities = new Set([null, 'P0', 'P1', 'P2', 'P3']);
const variantNames = new Set(['default', 'opus', 'argon', 'astra', 'kimi']);
const statuses = new Set(['unreviewed', 'keep', 'change', 'cut']);

export function normalizeReview(value) {
  if (!value || value.version !== 1 || !value.items || typeof value.items !== 'object' || Array.isArray(value.items)) throw new Error('Invalid kit review JSON');
  const items = {};
  for (const [id, item] of Object.entries(value.items)) items[id] = normalizeItem(item);
  return {version: 1, items};
}

export function normalizeItem(item) {
  if (!item || typeof item !== 'object' || !priorities.has(item.priority) || !statuses.has(item.status) || typeof item.notes !== 'string' || !Array.isArray(item.tags) || !item.tags.every((tag) => typeof tag === 'string') || typeof item.updated !== 'string' || Number.isNaN(Date.parse(item.updated))) throw new Error('Invalid kit review item');
  const variants = {};
  if (item.variants !== undefined) {
    if (!item.variants || typeof item.variants !== 'object' || Array.isArray(item.variants)) throw new Error('Invalid kit review item');
    for (const [name, level] of Object.entries(item.variants)) {
      if (!variantNames.has(name) || !priorities.has(level)) throw new Error('Invalid kit review item');
      if (level !== null) variants[name] = level;
    }
  }
  return {priority: item.priority, status: item.status, notes: item.notes, tags: item.tags, ...(Object.keys(variants).length ? {variants} : {}), updated: item.updated};
}

export function readReview(file) {
  if (!fs.existsSync(file)) return blank();
  return normalizeReview(JSON.parse(fs.readFileSync(file, 'utf8')));
}

export function mergeReviewItem(review, id, item) {
  if (typeof id !== 'string' || !id) throw new Error('A kit item id is required');
  const next = normalizeItem(item);
  const current = review.items[id];
  // Each item is independently resolved; an older tab may not clobber a newer edit.
  if (!current || Date.parse(next.updated) >= Date.parse(current.updated)) review.items[id] = next;
  return review;
}

export function writeReviewAtomically(file, review) {
  fs.mkdirSync(path.dirname(file), {recursive: true});
  const temp = `${file}.${process.pid}.${Date.now()}.tmp`;
  fs.writeFileSync(temp, `${JSON.stringify(review, null, 2)}\n`, 'utf8');
  fs.renameSync(temp, file);
}

export function createKitReviewMiddleware(file) {
  return (req, res, next) => {
    if (req.url !== '/api/kit-review') return next();
    try {
      if (req.method === 'GET') {
        res.setHeader('Content-Type', 'application/json');
        return res.end(JSON.stringify(readReview(file)));
      }
      if (req.method !== 'POST') { res.statusCode = 405; return res.end('Method not allowed'); }
      let body = '';
      req.on('data', (chunk) => { body += chunk; });
      req.on('end', () => {
        try {
          const {id, item} = JSON.parse(body);
          const review = mergeReviewItem(readReview(file), id, item);
          writeReviewAtomically(file, review);
          res.setHeader('Content-Type', 'application/json');
          res.end(JSON.stringify({ok: true, item: review.items[id]}));
        } catch (error) { res.statusCode = 400; res.end(error instanceof Error ? error.message : 'Invalid request'); }
      });
    } catch (error) { res.statusCode = 500; res.end(error instanceof Error ? error.message : 'Review store error'); }
  };
}
