import { escapeHtml } from '../utils/escape-html.js';
import { renderStateBadge } from './state-badge.js';

/**
 * Render a feature list item.
 * @param {{ feature: object, onClick?: function }} opts
 * @returns {string} HTML string
 */
export function renderFeatureListItem({ feature, onClick }) {
  const id = escapeHtml(feature.id);
  const name = escapeHtml(feature.name);
  const desc = escapeHtml(feature.description || 'No description');
  const date = escapeHtml(new Date(feature.updated_at).toLocaleDateString());
  return (
    `<li class="list-item feature-item" data-feature-id="${id}">` +
      `<div class="feature-item-main">` +
        `<span class="feature-item-name">${name}</span>` +
        `<span class="feature-item-desc">${desc}</span>` +
      `</div>` +
      `<div class="feature-item-meta">` +
        `${renderStateBadge(feature.state)}` +
        `<span class="feature-item-date">${date}</span>` +
      `</div>` +
    `</li>`
  );
}
