import { invoke } from '../services/tauri-bridge.js';
import { renderStatCard } from '../components/stat-card.js';
import { renderFeatureListItem } from '../components/feature-list-item.js';
import { showModal } from '../components/modal.js';
import { showToast } from '../components/toast.js';

/**
 * Dashboard view: stats cards + recent features + create button.
 * @param {HTMLElement} container
 */
export async function render(container) {
  container.innerHTML = '<div class="loading">Loading dashboard...</div>';

  try {
    const [stats, features] = await Promise.all([
      invoke('get_dashboard_stats'),
      invoke('list_features'),
    ]);

    const inProgress = (stats.features_by_state.implementing || 0)
      + (stats.features_by_state.planned || 0)
      + (stats.features_by_state.specified || 0)
      + (stats.features_by_state.researched || 0);
    const shipped = stats.features_by_state.shipped || 0;

    const recentFeatures = features.slice(0, 5);

    container.innerHTML = (
      `<div class="view-header">` +
        `<h2 class="view-title">Dashboard</h2>` +
        `<button class="btn btn-primary" id="dash-create-btn">+ Create Feature</button>` +
      `</div>` +
      `<div class="stats-grid">` +
        `${renderStatCard({ title: 'Total Features', value: stats.total_features, color: 'var(--accent)' })}` +
        `${renderStatCard({ title: 'In Progress', value: inProgress, color: 'var(--color-implementing)' })}` +
        `${renderStatCard({ title: 'Shipped', value: shipped, color: 'var(--color-shipped)' })}` +
        `${renderStatCard({ title: 'Total Work Packages', value: stats.total_work_packages, color: 'var(--accent)' })}` +
      `</div>` +
      `<div class="section">` +
        `<div class="section-header">` +
          `<h3 class="section-title">Recent Features</h3>` +
        `</div>` +
        (recentFeatures.length === 0
          ? `<div class="empty-state">No features yet. Create one to get started.</div>`
          : `<ul class="list">${recentFeatures.map(f =>
              renderFeatureListItem({ feature: f })
            ).join('')}</ul>`
        ) +
      `</div>`
    );

    // Wire create button
    document.getElementById('dash-create-btn').addEventListener('click', () => {
      openCreateModal();
    });

    // Wire feature item clicks
    container.querySelectorAll('.feature-item').forEach(el => {
      el.style.cursor = 'pointer';
      el.addEventListener('click', () => {
        const id = el.dataset.featureId;
        window.location.hash = `#feature/${id}`;
      });
    });

  } catch (err) {
    container.innerHTML = `<div class="error-state">Failed to load dashboard: ${String(err)}</div>`;
  }
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
        window.location.hash = '#features';
      } catch (err) {
        showToast({ message: `Failed to create feature: ${err}`, type: 'error' });
      }
    },
  });
}
