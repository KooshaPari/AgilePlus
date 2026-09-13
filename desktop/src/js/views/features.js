import { invoke } from '../services/tauri-bridge.js';
import { renderFeatureListItem } from '../components/feature-list-item.js';
import { renderStateBadge, getNextStates } from '../components/state-badge.js';
import { showModal } from '../components/modal.js';
import { showToast } from '../components/toast.js';

const ALL_STATES = [
  'created', 'specified', 'researched', 'planned',
  'implementing', 'validated', 'shipped', 'retrospected',
];

/**
 * Feature list view with state filter.
 * @param {HTMLElement} container
 */
export async function render(container) {
  container.innerHTML = '<div class="loading">Loading features...</div>';

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
        `<button class="btn btn-primary" id="feat-create-btn">+ Create Feature</button>` +
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

  document.getElementById('feat-create-btn').addEventListener('click', () => {
    openCreateModal();
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

function openCreateModal() {
  const formHtml = (
    `<div class="form-group">` +
      `<label class="form-label" for="feat-name">Feature Name</label>` +
      `<input class="form-input" type="text" id="feat-name" placeholder="Enter feature name" maxlength="255" />` +
    `</div>` +
    `<div class="form-group">` +
      `<label class="form-label" for="feat-desc">Description (optional)</label>` +
      `<textarea class="form-textarea" id="feat-desc" rows="3" placeholder="Describe the feature..."></textarea>` +
    `</div>`
  );

  showModal({
    title: 'Create Feature',
    content: formHtml,
    onConfirm: async () => {
      const name = document.getElementById('feat-name').value.trim();
      const description = document.getElementById('feat-desc').value.trim() || null;
      if (!name) {
        showToast({ message: 'Feature name is required', type: 'error' });
        return;
      }
      try {
        await invoke('create_feature', { name, description });
        showToast({ message: `Feature "${name}" created`, type: 'success' });
        // Re-render the list
        const features = await invoke('list_features');
        const container = document.getElementById('content');
        renderList(container, features);
      } catch (err) {
        showToast({ message: `Failed to create feature: ${err}`, type: 'error' });
      }
    },
  });
}
