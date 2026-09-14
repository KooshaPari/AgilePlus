import { invoke } from '../services/tauri-bridge.js';
import { renderStatCard } from '../components/stat-card.js';
import { renderFeatureListItem } from '../components/feature-list-item.js';

/**
 * Dashboard view: stats cards + recent features + create button.
 * @param {HTMLElement} container
 */
export async function render(container) {
  container.innerHTML = '<div class="loading">Loading dashboard...</div>';

  try {
    // Check if a project is loaded
    let repoPath = '';
    try {
      repoPath = await invoke('get_repo_path');
    } catch (_e) { /* ignore */ }

    if (!repoPath) {
      container.innerHTML = (
        `<div class="view-header">` +
          `<h2 class="view-title">AgilePlus Desktop</h2>` +
        `</div>` +
        `<div class="empty-state" style="padding: 4rem; text-align: center;">` +
          `<h3>No project open</h3>` +
          `<p>Open a project directory to get started.</p>` +
          `<button class="btn btn-primary" id="dash-open-btn">Open Project</button>` +
        `</div>`
      );
      document.getElementById('dash-open-btn')?.addEventListener('click', async () => {
        const path = prompt('Enter project directory path:', '');
        if (path) {
          try {
            await invoke('open_project', { path });
            window.location.hash = '#dashboard';
            window.location.reload();
          } catch (e) {
            alert(`Failed: ${e}`);
          }
        }
      });
      return;
    }

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
        `<div>` +
          `<button class="btn btn-secondary" id="dash-open-btn" title="Open a different project">Open Project</button>` +
        `</div>` +
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

    // Wire open project button
    document.getElementById('dash-open-btn')?.addEventListener('click', async () => {
      const path = prompt('Enter project directory path:', '');
      if (path) {
        try {
          await invoke('open_project', { path });
          window.location.hash = '#dashboard';
          window.location.reload();
        } catch (e) {
          alert(`Failed: ${e}`);
        }
      }
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


