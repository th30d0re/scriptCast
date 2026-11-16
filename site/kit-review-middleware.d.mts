export type KitReviewItem = { priority: 'P0'|'P1'|'P2'|'P3'|null; status: 'unreviewed'|'keep'|'change'|'cut'; notes: string; tags: string[]; updated: string };
export type KitReview = { version: 1; items: Record<string, KitReviewItem> };
export const createKitReviewMiddleware: any;
