import assert from 'node:assert/strict';
import {mergeReviewItem} from './kit-review-middleware.mjs';

const old = {priority: 'P2', status: 'keep', notes: 'old', tags: [], updated: '2026-01-01T00:00:00.000Z'};
const newer = {...old, notes: 'new', updated: '2026-01-02T00:00:00.000Z'};
const review = {version: 1, items: {CitationCard: old, DrawLine: old}};
mergeReviewItem(review, 'CitationCard', newer);
mergeReviewItem(review, 'CitationCard', old);
assert.equal(review.items.CitationCard.notes, 'new');
assert.equal(review.items.DrawLine.notes, 'old');
console.log('kit-review middleware merge: ok');
