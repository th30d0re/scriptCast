import {StrictMode, Suspense, lazy, useEffect, useState} from 'react';
import {createRoot} from 'react-dom/client';
import {ScrollPage} from './ScrollPage';
import './style.css';

// The player page pulls in remotion, so it loads only when someone opens it.
const PlayerPage = lazy(() => import('./PlayerPage'));

function useHash() {
  const [hash, setHash] = useState(location.hash);
  useEffect(() => {
    const onChange = () => setHash(location.hash);
    addEventListener('hashchange', onChange);
    return () => removeEventListener('hashchange', onChange);
  }, []);
  return hash;
}

function App() {
  const hash = useHash();
  return <>
    <nav className="nav">
      <a href="#" className={hash !== '#player' ? 'on' : ''}>Scroll</a>
      <a href="#player" className={hash === '#player' ? 'on' : ''}>Player</a>
    </nav>
    {hash === '#player' ? <Suspense fallback={null}><PlayerPage /></Suspense> : <ScrollPage />}
  </>;
}

createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>);
