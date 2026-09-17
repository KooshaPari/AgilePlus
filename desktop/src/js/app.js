/**
 * AgilePlus Desktop - App Bootstrap & Router
 * Hash-based SPA routing with view modules.
 */

import * as dashboard from './views/dashboard.js';
import * as features from './views/features.js';
import * as featureDetail from './views/feature-detail.js';
import * as adrs from './views/adrs.js';
import * as traces from './views/traces.js';

const NAV_ITEMS = [
  { id: 'dashboard', label: 'Dashboard', hash: '#dashboard' },
  { id: 'features', label: 'Features', hash: '#features' },
  { id: 'adrs', label: 'ADRs', hash: '#adrs' },
  { id: 'traces', label: 'Traces', hash: '#traces' },
];

/** Dismiss splash screen with fade. */
function dismissSplash() {
  const splash = document.getElementById('splash');
  const app = document.getElementById('app');
  if (splash) {
    splash.classList.add('fade-out');
    setTimeout(() => splash.remove(), 500);
  }
  if (app) app.classList.remove('hidden');
}

/** Update splash status text. */
function setSplashStatus(msg) {
  const el = document.getElementById('splash-status');
  if (el) el.textContent = msg;
}

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
      await adrs.render(content);
      break;
    case 'traces':
      await traces.render(content);
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
  setSplashStatus('Loading workspace...');

  // Build sidebar
  renderSidebar();

  // Small delay so splash is visible at least briefly
  await new Promise(r => setTimeout(r, 600));

  setSplashStatus('Ready');
  await new Promise(r => setTimeout(r, 200));

  // Fade splash, reveal app
  dismissSplash();

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
