import { invoke } from '../services/tauri-bridge.js';
import { escapeHtml } from '../utils/escape-html.js';

/**
 * ADR list view - browse Architecture Decision Records.
 * @param {HTMLElement} container
 */
export async function render(container) {
  container.innerHTML = '<div class="loading">Loading ADRs...</div>';

  try {
    // Get repo path from backend, then fetch ADRs
    const repoPath = await invoke('get_repo_path');
    const adrs = await invoke('list_adrs', { repoPath: repoPath || '.' });
    renderList(container, adrs);
  } catch (err) {
    container.innerHTML = (
      `<div class="view-header"><h2 class="view-title">ADRs</h2></div>` +
      `<div class="error-state">Failed to load ADRs: ${escapeHtml(String(err))}</div>`
    );
  }
}

function renderList(container, adrs) {
  container.innerHTML = (
    `<div class="view-header">` +
      `<h2 class="view-title">Architecture Decision Records</h2>` +
      `<span class="view-count">${adrs.length} ADRs</span>` +
    `</div>` +
    `<div class="list-container">` +
      `<ul class="list" id="adr-list"></ul>` +
    `</div>` +
    `<div class="detail-panel" id="adr-detail" hidden>` +
      `<div class="detail-header">` +
        `<h3 id="adr-detail-title"></h3>` +
        `<button class="btn btn-sm" id="adr-close-btn">Close</button>` +
      `</div>` +
      `<pre class="detail-content" id="adr-detail-content"></pre>` +
    `</div>`
  );

  const listEl = document.getElementById('adr-list');

  if (adrs.length === 0) {
    listEl.innerHTML = '<div class="empty-state">No ADRs found. Create docs/adr/ directory with .md files.</div>';
    return;
  }

  listEl.innerHTML = adrs.map(adr => (
    `<li class="list-item adr-item" data-adr-id="${escapeHtml(adr.id)}">` +
      `<div class="item-main">` +
        `<span class="item-id">${escapeHtml(adr.id)}</span>` +
        `<span class="item-title">${escapeHtml(adr.title)}</span>` +
      `</div>` +
      `<span class="item-status">${escapeHtml(adr.status)}</span>` +
    `</li>`
  )).join('');

  // Wire click to show detail
  listEl.querySelectorAll('.adr-item').forEach(el => {
    el.style.cursor = 'pointer';
    el.addEventListener('click', async () => {
      const id = el.dataset.adrId;
      await showAdrDetail(id);
    });
  });

  document.getElementById('adr-close-btn').addEventListener('click', () => {
    document.getElementById('adr-detail').hidden = true;
  });
}

async function showAdrDetail(id) {
  const detailPanel = document.getElementById('adr-detail');
  const titleEl = document.getElementById('adr-detail-title');
  const contentEl = document.getElementById('adr-detail-content');

  try {
    const repoPath = await invoke('get_repo_path');
    const content = await invoke('read_adr', { repoPath: repoPath || '.', id });
    titleEl.textContent = `ADR ${id}`;
    contentEl.textContent = content;
    detailPanel.hidden = false;
  } catch (err) {
    titleEl.textContent = `ADR ${id}`;
    contentEl.textContent = `Failed to load: ${err}`;
    detailPanel.hidden = false;
  }
}
