// Narrow declaration for the sole server API used by the offline copy check.
// @types/react-dom is outside this phase's package-install allowlist.
declare module 'react-dom/server' {
  export function renderToStaticMarkup(node: import('react').ReactNode): string;
}
