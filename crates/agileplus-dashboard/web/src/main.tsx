import React from 'react';
import ReactDOM from 'react-dom/client';
import { useAgilePlusStore } from './stores/agileplus';
import './styles/globals.css';

function App() {
  const workPackages = useAgilePlusStore((state) => state.workPackages);
  const loading = useAgilePlusStore((state) => state.loading);

  return (
    <main className="container" style={{ padding: '2rem 0' }}>
      <section
        style={{
          background: 'white',
          borderRadius: 'var(--border-radius-lg)',
          padding: '2rem',
          boxShadow: 'var(--shadow-md)',
          border: '1px solid rgba(17, 24, 39, 0.08)',
        }}
      >
        <p style={{ margin: 0, color: 'var(--color-neutral-500)' }}>AgilePlus</p>
        <h1 style={{ marginTop: '0.25rem' }}>Dashboard</h1>
        <p>
          {loading
            ? 'Loading work packages...'
            : `Work packages loaded: ${workPackages.length}`}
        </p>
      </section>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
