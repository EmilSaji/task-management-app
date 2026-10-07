import { createRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { App } from './App';
import { AuthProvider } from './context/AuthContext';
import './index.css';

// Note: <StrictMode> is intentionally omitted. In development it mounts every
// component twice, which would fire GET /tasks/view-my-tasks twice on page
// load and make the very first render already show cache.hit = true.
createRoot(document.getElementById('root')!).render(
  <BrowserRouter>
    <AuthProvider>
      <App />
    </AuthProvider>
  </BrowserRouter>,
);
