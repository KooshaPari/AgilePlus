import { invoke } from '../services/tauri-bridge.js';
import { renderStateBadge, renderStateMachine, getNextState } from '../components/state-badge.js';
import { showToast } from '../components/toast.js';
import { escapeHtml } from '../utils/escape-html.js';

/**
 * Feature detail view with tabs: Overview, Work Packages, Evidence.
 * @param {HTMLElement} container
 * @param {string} featureId
 */
export async function render(container, featureId) {
  container.innerHTML = '<div class="loading">Loading feature...</div>';

  try {
    const feature = await invoke('get_feature', { id: featureId });
    if (!feature) {
      container.innerHTML = '<div class="error-state">Feature not found.</div>';
      return;
    }
    renderDetail(container, feature);
  } catch (err) {
    container.innerHTML = `<div class="error-state">Failed to load feature: ${String(err)}</div>`;
  }
}

function renderDetail(container, feature) {
  const name = escapeHtml(feature.name);
  const desc = escapeHtml(feature.description || 'No description');
  const created = escapeHtml(new Date(feature.created_at).toLocaleString());
  const updated = escapeHtml(new Date(feature.updated_at).toLocaleString());
  const nextState = getNextState(feature.state);

  container.innerHTML = (
    `<div class="view-header">` +
      `<div class="detail-header-row">` +
        `<div>` +
          `<h2 class="view-title">${name}</h2>` +
          `<div class="detail-meta">` +
            `${renderStateBadge(feature.state)}` +
            `<span class="detail-date">Created: ${created}</span>` +
            `<span class="detail-date">Updated: ${updated}</span>` +
          `</div>` +
        `</div>` +
        `<div class="view-actions">` +
          `<button class="btn btn-back" id="detail-back-btn">&larr; Back</button>` +
          (nextState
            ? `<button class="btn btn-primary" id="detail-advance-btn">Advance to ${escapeHtml(nextState)}</button>`
            : '') +
        `</div>` +
      `</div>` +
    `</div>` +
    `<div class="detail-tabs" role="tablist">` +
      `<button class="tab active" data-tab="overview" role="tab">Overview</button>` +
      `<button class="tab" data-tab="work-packages" role="tab">Work Packages</button>` +
      `<button class="tab" data-tab="evidence" role="tab">Evidence</button>` +
    `</div>` +
    `<div class="detail-content" id="detail-tab-content"></div>`
  );

  // Wire tabs
  const tabs = container.querySelectorAll('.tab');
  const tabContent = document.getElementById('detail-tab-content');
  let activeTab = 'overview';

  function switchTab(tab) {
    activeTab = tab;
    tabs.forEach(t => t.classList.toggle('active', t.dataset.tab === tab));
    renderTab(tabContent, feature);
  }

  tabs.forEach(t => {
    t.addEventListener('click', () => switchTab(t.dataset.tab));
  });

  // Wire back button
  document.getElementById('detail-back-btn').addEventListener('click', () => {
    window.location.hash = '#features';
  });

  // Wire advance button
  const advanceBtn = document.getElementById('detail-advance-btn');
  if (advanceBtn) {
    advanceBtn.addEventListener('click', async () => {
      const next = getNextState(feature.state);
      if (!next) return;
      try {
        const updated = await invoke('update_feature_state', { id: feature.id, new_state: next });
        showToast({ message: `Advanced to "${next}"`, type: 'success' });
        renderDetail(container, updated);
      } catch (err) {
        showToast({ message: `Failed to advance: ${err}`, type: 'error' });
      }
    });
  }

  renderTab(tabContent, feature);
}

function renderTab(tabContent, feature) {
  const activeTab = document.querySelector('.detail-tabs .tab.active');
  const tab = activeTab ? activeTab.dataset.tab : 'overview';

  if (tab === 'overview') {
    tabContent.innerHTML = (
      `<div class="tab-panel">` +
        `<h3>Description</h3>` +
        `<p class="detail-description">${escapeHtml(feature.description || 'No description provided.')}</p>` +
        `<h3>State Progression</h3>` +
        `${renderStateMachine(feature.state)}` +
      `</div>`
    );
  } else if (tab === 'work-packages') {
    renderWorkPackagesTab(tabContent, feature);
  } else if (tab === 'evidence') {
    renderEvidenceTab(tabContent, feature);
  }
}

async function renderWorkPackagesTab(tabContent, feature) {
  tabContent.innerHTML = '<div class="loading">Loading work packages...</div>';
  try {
    const wp = await invoke('list_work_packages', { feature_id: feature.id });
    if (wp.length === 0) {
      tabContent.innerHTML = '<div class="empty-state">No work packages yet.</div>';
      return;
    }
    tabContent.innerHTML = (
      `<ul class="list">` +
        wp.map(w => (
          `<li class="list-item wp-item">` +
            `<div class="wp-main">` +
              `<span class="wp-name">${escapeHtml(w.name)}</span>` +
              `<span class="wp-desc">${escapeHtml(w.description || '')}</span>` +
            `</div>` +
            `<div class="wp-meta">` +
              `${renderStateBadge(w.state)}` +
            `</div>` +
          `</li>`
        )).join('') +
      `</ul>`
    );
  } catch (err) {
    tabContent.innerHTML = `<div class="error-state">Failed to load work packages: ${String(err)}</div>`;
  }
}

async function renderEvidenceTab(tabContent, feature) {
  tabContent.innerHTML = '<div class="loading">Loading evidence...</div>';
  try {
    const evidence = await invoke('list_evidence', { feature_id: feature.id });
    if (evidence.length === 0) {
      tabContent.innerHTML = '<div class="empty-state">No evidence collected yet.</div>';
      return;
    }
    tabContent.innerHTML = (
      `<ul class="list">` +
        evidence.map(e => (
          `<li class="list-item evidence-item">` +
            `<div class="evidence-main">` +
              `<span class="evidence-type">${escapeHtml(e.evidence_type)}</span>` +
              `<span class="evidence-content">${escapeHtml(e.content)}</span>` +
            `</div>` +
            `<span class="evidence-date">${escapeHtml(new Date(e.created_at).toLocaleDateString())}</span>` +
          `</li>`
        )).join('') +
      `</ul>`
    );
  } catch (err) {
    tabContent.innerHTML = `<div class="error-state">Failed to load evidence: ${String(err)}</div>`;
  }
}
