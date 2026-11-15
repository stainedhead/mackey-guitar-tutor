import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { installMockHooks } from './mocks/mockTuner';
import './theme/tokens.css';
import './App.css';

if (import.meta.env.MODE === 'mock') installMockHooks(window);

const root = document.getElementById('root');
if (!root) {
  throw new Error('index.html has no #root element');
}
createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
