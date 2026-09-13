import { invoke } from '../services/tauri-bridge.js';
import { escapeHtml } from '../utils/escape-html.js';

/**
 * Trace list view - browse traceability files.
 * @param {HTMLElement} container
 */
export async function render(container) {
  container.innerHTML = '<div class="loading">Loading traces...</div>';

  try {
    const repoPath = await invoke('get_repo_path');
    const traces = await invoke('list_traces', { repoPath: repoPath || '.' });
    renderList(container, traces);
  } catch (err) {
    container.innerHTML = (
      `<div class="view-header"><h2 class="view-title">Traces</h2></div>` +
      `<div class="error-state">Failed to load traces: ${escapeHtml(String(err))}</div>`
    );
  }
}

function renderList(container, traces) {
  container.innerHTML = (
    `<div class="view-header">` +
      `<h2 class="view-title">Traces</h2>` +
      `<span class="view-count">${traces.length} traces</span>` +
    `</div>` +
    `<div class="list-container">` +
      `<ul class="list" id="trace-list"></ul>` +
    `</div>` +
    `<div class="detail-panel" id="trace-detail" hidden>` +
      `<div class="detail-header">` +
        `<h3 id="trace-detail-title"></h3>` +
        `<button class="btn btn-sm" id="trace-close-btn">Close</button>` +
      `</div>` +
      `<pre class="detail-content" id="trace-detail-content"></pre>` +
    `</div>`
  );

  const listEl = document.getElementById('trace-list');

  if (traces.length === 0) {
    listEl.innerHTML = '<div class="empty-state">No traces found. Create traces/ directory with .jsonl or .md files.</div>';
    return;
  }

  listEl.innerHTML = traces.map(trace => (
    `<li class="list-item trace-item" data-trace-id="${escapeHtml(trace.id)}">` +
      `<div class="item-main">` +
        `<span class="item-id">${escapeHtml(trace.id)}</span>` +
      `</div>` +
      `<span class="item-type badge badge-${escapeHtml(trace.kind)}">${escapeHtml(trace.kind)}</span>` +
    `</li>`
  )).join('');

  // Wire click to show detail
  listEl.querySelectorAll('.trace-item').forEach(el => {
    el.style.cursor = 'pointer';
    el.addEventListener('click', async () => {
      const id = el.dataset.traceId;
      await showTraceDetail(id);
    });
  });

  document.getElementById('trace-close-btn').addEventListener('click', () => {
    document.getElementById('trace-detail').hidden = true;
  });
}

async function showTraceDetail(id) {
  const detailPanel = document.getElementById('trace-detail');
  const titleEl = document.getElementById('trace-detail-title');
  const contentEl = document.getElementById('trace-detail-content');

  try {
    const repoPath = await invoke('get_repo_path');
    const content = await invoke('read_trace', { repoPath: repoPath || '.', id });
    titleEl.textContent = id;
    // Truncate large content
    const display = content.length > 10000
      ? content.substring(0, 10000) + '\n... (truncated)'
      : content;
    contentEl.textContent = display;
    detailPanel.hidden = false;
  } catch (err) {
    titleEl.textContent = id;
    contentEl.textContent = `Failed to load: ${err}`;
    detailPanel.hidden = false;
  }
}
