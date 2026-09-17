import { invoke } from '../services/tauri-bridge.js';
import { renderFeatureListItem } from '../components/feature-list-item.js';
import { renderStateBadge, getNextStates } from '../components/state-badge.js';

const ALL_STATES = [
  'created', 'specified', 'researched', 'planned',
  'implementing', 'validated', 'shipped', 'retrospected',
];

/**
 * Feature list view with state filter.
 * @param {HTMLElement} container
 */
export async function render(container) {
  container.innerHTML = '<div class="loading"><div class="spinner spinner-inline"></div> Loading features...</div>';

  try {
    const features = await invoke('list_features');
    renderList(container, features);
  } catch (err) {
    container.innerHTML = `<div class="error-state">Failed to load features: ${String(err)}</div>`;
  }
}

function renderList(container, features) {
  const filterOptions = ALL_STATES.map(s =>
    `<option value="${s}">${s.charAt(0).toUpperCase() + s.slice(1)}</option>`
  ).join('');

  container.innerHTML = (
    `<div class="view-header">` +
      `<h2 class="view-title">Features</h2>` +
      `<div class="view-actions">` +
        `<select class="form-select" id="state-filter">` +
          `<option value="">All States</option>` +
          `${filterOptions}` +
        `</select>` +
      `</div>` +
    `</div>` +
    `<div class="list-container">` +
      `<ul class="list" id="feature-list"></ul>` +
    `</div>`
  );

  const listEl = document.getElementById('feature-list');

  function renderFiltered(stateFilter) {
    const filtered = stateFilter
      ? features.filter(f => f.state === stateFilter)
      : features;

    if (filtered.length === 0) {
      listEl.innerHTML = '<div class="empty-state">No features found.</div>';
    } else {
      listEl.innerHTML = filtered.map(f =>
        renderFeatureListItem({ feature: f })
      ).join('');
      wireClicks(listEl);
    }
  }

  renderFiltered('');

  document.getElementById('state-filter').addEventListener('change', (e) => {
    renderFiltered(e.target.value);
  });

}

function wireClicks(listEl) {
  listEl.querySelectorAll('.feature-item').forEach(el => {
    el.style.cursor = 'pointer';
    el.addEventListener('click', () => {
      const id = el.dataset.featureId;
      window.location.hash = `#feature/${id}`;
    });
  });
}

