// Minimal declaration for the sole ReactDOM API used here; @types/react-dom is not installed.
declare module 'react-dom/client' {
  export function createRoot(container: Element | DocumentFragment): {render(children: import('react').ReactNode): void; unmount(): void};
}
