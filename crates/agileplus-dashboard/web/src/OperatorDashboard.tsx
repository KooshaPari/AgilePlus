import { useEffect, useState } from 'react';
import { ApiError, api, describeError, featurePath, pendingKey, prepareRequest, readPending } from './operator-api';
import type { AcceptanceRequest, Feature, Governance, Outcome, Receipt, WorkPackage } from './operator-api';

const button = 'rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium hover:bg-slate-100 disabled:opacity-50';

export function OperatorDashboard() {
  const [features, setFeatures] = useState<Feature[]>([]);
  const [selected, setSelected] = useState<string>('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    const controller = new AbortController();
    setLoading(true); setError('');
    api<Feature[]>('/features', { signal: controller.signal }).then((data) => {
      if (!Array.isArray(data)) throw new Error('Invalid feature response from backend.');
      setFeatures(data);
      setSelected((slug) => data.some((feature) => feature.slug === slug) ? slug : (data[0]?.slug ?? ''));
    }).catch((cause: unknown) => {
      if (!controller.signal.aborted) { setError(describeError(cause)); setFeatures([]); }
    }).finally(() => { if (!controller.signal.aborted) setLoading(false); });
    return () => controller.abort();
  }, [revision]);
  const feature = features.find((item) => item.slug === selected);

  return <div className="min-h-screen bg-slate-50 text-slate-900">
    <header className="border-b border-slate-200 bg-white px-6 py-5">
      <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4">
        <div><p className="text-xs font-semibold uppercase tracking-widest text-teal-700">Private operator alpha</p>
          <h1 className="text-2xl font-semibold">AgilePlus</h1></div>
        <button className={button} disabled={loading} onClick={() => setRevision((value) => value + 1)}>Refresh backend</button>
      </div>
    </header>
    <main className="mx-auto grid max-w-6xl gap-6 p-6 md:grid-cols-[260px_1fr]">
      <aside aria-label="Features" className="rounded-xl border border-slate-200 bg-white p-4">
        <h2 className="mb-4 font-semibold">Features</h2>
        {loading ? <p role="status">Loading backend…</p> : null}
        {error ? <p role="alert" className="text-sm text-red-700">{error}</p> : null}
        {!loading && !error && features.length === 0 ? <p>No features in this backend.</p> : null}
        <ul className="space-y-2">{features.map((item) => <li key={item.id}>
          <button aria-pressed={selected === item.slug} onClick={() => setSelected(item.slug)}
            className={`w-full rounded-lg p-3 text-left ${selected === item.slug ? 'bg-teal-50 ring-1 ring-teal-600' : 'hover:bg-slate-50'}`}>
            <span className="block font-medium">{item.name}</span><span className="text-xs text-slate-600">{item.state}</span>
          </button></li>)}</ul>
      </aside>
      {feature && !loading && !error ? <FeatureDetail key={`${feature.id}:${revision}`} feature={feature} /> :
        <section className="rounded-xl border border-slate-200 bg-white p-6"><h2 className="font-semibold">Development acceptance</h2>
          <p className="mt-2 text-sm text-slate-600">Select a feature from the connected backend to inspect its work and acceptance history.</p></section>}
    </main>
  </div>;
}

function FeatureDetail({ feature }: { feature: Feature }) {
  const [current, setCurrent] = useState(feature);
  const [wps, setWps] = useState<WorkPackage[]>([]);
  const [governance, setGovernance] = useState<Governance | null>(null);
  const [receipt, setReceipt] = useState<Receipt | null>(null);
  const [pending, setPending] = useState<AcceptanceRequest | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState('');
  const [canReset, setCanReset] = useState(false);
  const [notice, setNotice] = useState('');
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    const path = featurePath(feature.slug);
    async function optional<T>(suffix: string): Promise<T | null> {
      try { return await api<T>(path + suffix, { signal: controller.signal }); }
      catch (cause) { if (cause instanceof ApiError && cause.status === 404) return null; throw cause; }
    }
    setLoading(true); setReady(false); setError('');
    Promise.all([
      api<Feature>(path, { signal: controller.signal }),
      api<WorkPackage[]>(path + '/work-packages', { signal: controller.signal }),
      optional<Governance>('/governance'), optional<Receipt>('/acceptance-receipt'),
    ]).then(([detail, packages, contract, decision]) => {
      if (controller.signal.aborted) return;
      if (!Array.isArray(packages) || detail.id !== feature.id || (decision && decision.feature_id !== detail.id)) {
        throw new Error('Backend returned mismatched feature data.');
      }
      setCurrent(detail); setWps(packages); setGovernance(contract); setReceipt(decision);
      const saved = readPending(detail.id);
      if (decision && saved?.request_id === decision.request_id) {
        localStorage.removeItem(pendingKey(detail.id)); setPending(null);
      } else { setPending(saved); }
      setReady(true);
    }).catch((cause: unknown) => { if (!controller.signal.aborted) setError(describeError(cause)); })
      .finally(() => { if (!controller.signal.aborted) setLoading(false); });
    return () => controller.abort();
  }, [feature.id, feature.slug, revision]);

  async function accept() {
    if (!governance || busy) return;
    setBusy(true); setError(''); setNotice(''); setCanReset(false);
    try {
      const request = prepareRequest(current.id, governance.version);
      setPending(request);
      const outcome = await api<Outcome>(featurePath(current.slug) + '/accept', { method: 'POST', body: JSON.stringify(request) });
      if (outcome.receipt?.feature_id !== current.id || outcome.receipt.request_id !== request.request_id) {
        throw new Error('Acceptance response did not match the saved request. Retry the same request to reconcile.');
      }
      setReceipt(outcome.receipt);
      localStorage.removeItem(pendingKey(current.id)); setPending(null);
      setNotice(outcome.replayed ? 'Recovered the original committed receipt.' : 'Acceptance committed.');
      setRevision((value) => value + 1);
    } catch (cause) {
      setError(describeError(cause));
      setCanReset(cause instanceof ApiError && [400, 409, 422].includes(cause.status));
    } finally { setBusy(false); }
  }

  function resetRequest() {
    try {
      localStorage.removeItem(pendingKey(current.id)); setPending(null); setCanReset(false);
      setNotice('Rejected request cleared. Refreshing the current governance contract.');
      setRevision((value) => value + 1);
    } catch (cause) { setError(describeError(cause)); }
  }

  return <section className="space-y-5" aria-busy={loading || busy}>
    <div className="rounded-xl border border-slate-200 bg-white p-6">
      <div className="flex flex-wrap items-start justify-between gap-3"><div>
        <p className="text-xs uppercase tracking-widest text-slate-500">{current.slug}</p>
        <h2 className="mt-1 text-xl font-semibold">{current.name}</h2></div>
        <span className="rounded-full bg-slate-100 px-3 py-1 text-sm">{current.state}</span></div>
      <p className="mt-3 text-sm text-slate-600">Target branch: {current.target_branch}</p>
      {loading ? <p role="status" className="mt-4">Loading work and acceptance history…</p> : null}
      {error ? <p role="alert" className="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-800">{error}</p> : null}
      {notice ? <p role="status" className="mt-4 text-sm text-teal-800">{notice}</p> : null}
      {!loading ? <div className="mt-5 flex flex-wrap gap-3">
        <button className={button} disabled={busy} onClick={() => setRevision((value) => value + 1)}>Refresh feature</button>
        <button className={`${button} border-teal-600 text-teal-800`} onClick={() => void accept()}
          disabled={busy || !ready || !governance || (current.state !== 'implementing' && !pending)}>
          {busy ? 'Checking acceptance…' : pending ? 'Retry acceptance' : 'Accept feature'}
        </button>
        {canReset ? <button className={button} disabled={busy} onClick={resetRequest}>Clear rejected request</button> : null}
      </div> : null}
      {pending ? <p className="mt-3 break-all text-xs text-slate-600">Pending request: {pending.request_id}. Retries preserve this identity and governance version.</p> : null}
      {!loading && !governance ? <p className="mt-3 text-sm text-amber-800">No governance contract is configured; acceptance is unavailable.</p> : null}
    </div>
    {!loading && ready ? <div className="rounded-xl border border-slate-200 bg-white p-6">
      <h3 className="font-semibold">Work packages</h3>
      {wps.length === 0 ? <p className="mt-3 text-sm text-slate-600">No work packages.</p> :
        <ul className="mt-3 divide-y divide-slate-100">{wps.map((wp) => <li key={wp.id} className="py-3">
          <div className="flex justify-between gap-3"><span className="font-medium">WP{String(wp.sequence).padStart(2, '0')} · {wp.title}</span>
            <span className="text-sm text-slate-600">{wp.state}</span></div><p className="mt-1 whitespace-pre-wrap text-sm text-slate-600">{wp.acceptance_criteria}</p>
        </li>)}</ul>}
    </div> : null}
    {!loading && ready ? <div className="rounded-xl border border-slate-200 bg-white p-6">
      <h3 className="font-semibold">Acceptance receipt</h3>
      <p className="mt-2 text-sm text-slate-600">A historical development decision. It does not establish current product satisfaction or promotion.</p>
      {receipt ? <><dl className="mt-4 space-y-2 text-sm">
        <div><dt className="text-slate-500">Request</dt><dd className="break-all font-mono">{receipt.request_id}</dd></div>
        <div><dt className="text-slate-500">Committed</dt><dd>{receipt.committed_at}</dd></div>
        <div><dt className="text-slate-500">Governance</dt><dd>Version {receipt.governance_version} · Event {receipt.event_id}</dd></div>
      </dl><ul className="mt-4 space-y-2">{receipt.accepted_candidates.map((candidate) => <li key={candidate.wp_id} className="break-all rounded-lg bg-slate-50 p-3 font-mono text-xs">
        WP {candidate.wp_id}: {candidate.candidate_ref}<br />Evaluation: {candidate.evaluation_id}
      </li>)}</ul></> : <p className="mt-4 text-sm text-slate-600">No committed acceptance receipt.</p>}
    </div> : null}
  </section>;
}
