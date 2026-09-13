import { escapeHtml } from '../utils/escape-html.js';

/**
 * Render a dashboard stat card.
 * @param {{ title: string, value: number|string, color?: string }} opts
 * @returns {string} HTML string
 */
export function renderStatCard({ title, value, color }) {
  const style = color ? ` style="color:${escapeHtml(color)}"` : '';
  return (
    `<div class="stat-card">` +
      `<div class="stat-card-title">${escapeHtml(title)}</div>` +
      `<div class="stat-card-value"${style}>${escapeHtml(String(value))}</div>` +
    `</div>`
  );
}
