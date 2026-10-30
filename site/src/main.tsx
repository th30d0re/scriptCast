import {StrictMode, Suspense, lazy, useEffect, useState} from 'react';
import {createRoot} from 'react-dom/client';
import {LANDSCAPE, PORTRAIT} from '@scriptcast/cards';
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
  const [landscape, setLandscape] = useState(false);
  const format = landscape ? LANDSCAPE : PORTRAIT;
  return <>
    <nav className="nav">
      <a href="#" className={hash !== '#player' ? 'on' : ''}>Scroll</a>
      <a href="#player" className={hash === '#player' ? 'on' : ''}>Player</a>
      <span className="sep" />
      <button type="button" className={landscape ? '' : 'on'} aria-pressed={!landscape} onClick={() => setLandscape(false)}>9:16</button>
      <button type="button" className={landscape ? 'on' : ''} aria-pressed={landscape} onClick={() => setLandscape(true)}>16:9</button>
    </nav>
    {hash === '#player' ? <Suspense fallback={null}><PlayerPage format={format} /></Suspense> : <ScrollPage format={format} />}
  </>;
}

createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>);
