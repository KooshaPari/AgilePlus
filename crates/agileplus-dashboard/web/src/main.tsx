import React, { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import axios from 'axios';
import { useAgilePlusStore } from './stores/agileplus';
import './styles/globals.css';

interface Epic {
  id: number;
  title: string;
  status: string;
  requirement_id: string | null;
}

interface Story {
  id: number;
  epic_id: number | null;
  title: string;
  status: string;
  requirement_id: string | null;
}

function App() {
  const workPackages = useAgilePlusStore((state) => state.workPackages);
  const loading = useAgilePlusStore((state) => state.loading);
  const setWorkPackages = useAgilePlusStore((state) => state.setWorkPackages);
  const setLoading = useAgilePlusStore((state) => state.setLoading);

  const [epics, setEpics] = useState<Epic[]>([]);
  const [stories, setStories] = useState<Story[]>([]);
  const [epicStoriesLoading, setEpicStoriesLoading] = useState(true);

  // Fetch work packages (in-memory store)
  useEffect(() => {
    setLoading(true);
    axios
      .get('/api/dashboard/work-packages.json')
      .then((res) => {
        const data = res.data as { work_packages: any[] };
        setWorkPackages(
          (data.work_packages ?? []).map((wp: any) => ({
            id: String(wp.id),
            title: wp.title ?? '(untitled)',
            status: wp.status ?? 'planned',
            priority: wp.priority ?? 'medium',
            assignee: wp.assignee ?? undefined,
          }))
        );
      })
      .catch(() => {
        // backend not available — store stays empty
      })
      .finally(() => setLoading(false));
  }, [setWorkPackages, setLoading]);

  // Fetch epics + stories from SQLite
  useEffect(() => {
    setEpicStoriesLoading(true);
    axios
      .get('/api/dashboard/epics-stories.json')
      .then((res) => {
        const data = res.data as { epics: Epic[]; stories: Story[] };
        setEpics(data.epics ?? []);
        setStories(data.stories ?? []);
      })
      .catch(() => {
        // backend not available
      })
      .finally(() => setEpicStoriesLoading(false));
  }, []);

  return (
    <main className="container" style={{ padding: '2rem 0' }}>
      <section
        style={{
          background: 'white',
          borderRadius: 'var(--border-radius-lg)',
          padding: '2rem',
          boxShadow: 'var(--shadow-md)',
          border: '1px solid rgba(17, 24, 39, 0.08)',
          marginBottom: '1.5rem',
        }}
      >
        <p style={{ margin: 0, color: 'var(--color-neutral-500)' }}>AgilePlus</p>
        <h1 style={{ marginTop: '0.25rem' }}>Dashboard</h1>
        <p>
          {epicStoriesLoading
            ? 'Loading epics & stories...'
            : `${epics.length} Epic${epics.length !== 1 ? 's' : ''} · ${stories.length} ${stories.length !== 1 ? 'Stories' : 'Story'}`}
        </p>
      </section>

      {!epicStoriesLoading && epics.length > 0 && (
        <section
          style={{
            background: 'white',
            borderRadius: 'var(--border-radius-lg)',
            padding: '2rem',
            boxShadow: 'var(--shadow-md)',
            border: '1px solid rgba(17, 24, 39, 0.08)',
          }}
        >
          <h2 style={{ marginTop: 0 }}>Epics &amp; Stories</h2>
          {epics.map((epic) => {
            const epicStories = stories.filter((s) => s.epic_id === epic.id);
            return (
              <details key={epic.id} style={{ marginBottom: '1rem' }}>
                <summary
                  style={{
                    cursor: 'pointer',
                    fontWeight: 600,
                    fontSize: '1rem',
                    padding: '0.5rem 0',
                    borderBottom: '1px solid rgba(17,24,39,0.08)',
                  }}
                >
                  {epic.title}
                  <span
                    style={{
                      marginLeft: '0.75rem',
                      fontSize: '0.8rem',
                      color: 'var(--color-neutral-500)',
                      fontWeight: 400,
                    }}
                  >
                    {epicStories.length} stories · {epic.status}
                  </span>
                </summary>
                <ul style={{ marginTop: '0.5rem', paddingLeft: '1.5rem' }}>
                  {epicStories.map((story) => (
                    <li key={story.id} style={{ marginBottom: '0.25rem', fontSize: '0.9rem' }}>
                      <span style={{ color: 'var(--color-neutral-700)' }}>{story.title}</span>
                      <span
                        style={{
                          marginLeft: '0.5rem',
                          fontSize: '0.75rem',
                          color: story.status === 'Done' ? 'green' : 'var(--color-neutral-400)',
                        }}
                      >
                        [{story.status}]
                      </span>
                    </li>
                  ))}
                </ul>
              </details>
            );
          })}
        </section>
      )}
    </main>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
