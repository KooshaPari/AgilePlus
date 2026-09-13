/**
 * AgilePlus Desktop - App Bootstrap & Router
 * Hash-based SPA routing with view modules.
 */

import * as dashboard from './views/dashboard.js';
import * as features from './views/features.js';
import * as featureDetail from './views/feature-detail.js';

const NAV_ITEMS = [
  { id: 'dashboard', label: 'Dashboard', hash: '#dashboard' },
  { id: 'features', label: 'Features', hash: '#features' },
  { id: 'adrs', label: 'ADRs', hash: '#adrs' },
  { id: 'traces', label: 'Traces', hash: '#traces' },
];

/** Get the content container element. */
function getContentEl() {
  return document.getElementById('content');
}

/** Update active nav item highlighting. */
function updateNav(viewId) {
  document.querySelectorAll('.sidebar .nav-item').forEach(el => {
    el.classList.toggle('active', el.dataset.view === viewId);
  });
}

/** Parse the hash into { view, param }. */
function parseHash() {
  const hash = window.location.hash || '#dashboard';
  const clean = hash.replace(/^#\/?/, '');

  if (clean.startsWith('feature/')) {
    const param = clean.split('/')[1] || '';
    return { view: 'feature', param };
  }

  const known = ['dashboard', 'features', 'adrs', 'traces'];
  if (known.includes(clean)) {
    return { view: clean, param: null };
  }

  return { view: 'dashboard', param: null };
}

/** Render the active view into the content container. */
async function route() {
  const content = getContentEl();
  const { view, param } = parseHash();

  updateNav(view);

  switch (view) {
    case 'dashboard':
      await dashboard.render(content);
      break;
    case 'features':
      await features.render(content);
      break;
    case 'feature':
      await featureDetail.render(content, param);
      break;
    case 'adrs':
      content.innerHTML = (
        `<div class="view-header"><h2 class="view-title">ADRs</h2></div>` +
        `<div class="empty-state">ADR browsing coming soon. Requires filesystem command.</div>`
      );
      break;
    case 'traces':
      content.innerHTML = (
        `<div class="view-header"><h2 class="view-title">Traces</h2></div>` +
        `<div class="empty-state">Trace browsing coming soon. Requires filesystem command.</div>`
      );
      break;
    default:
      await dashboard.render(content);
  }
}

/** Build sidebar nav HTML. */
function renderSidebar() {
  const sidebar = document.getElementById('sidebar');
  sidebar.innerHTML = NAV_ITEMS.map(item => (
    `<a class="nav-item" href="${item.hash}" data-view="${item.id}">${item.label}</a>`
  )).join('');
}

/** Bootstrap the app. */
async function init() {
  renderSidebar();

  // Route on hash change
  window.addEventListener('hashchange', route);

  // Route on initial load
  if (!window.location.hash) {
    window.location.hash = '#dashboard';
  } else {
    await route();
  }
}

// Start when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
