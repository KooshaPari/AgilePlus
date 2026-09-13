import { escapeHtml } from '../utils/escape-html.js';

/**
 * Show a toast notification (auto-dismiss after 3s).
 * @param {{ message: string, type?: 'info'|'success'|'error'|'warning' }} opts
 */
export function showToast({ message, type = 'info' }) {
  let container = document.getElementById('ap-toast-container');
  if (!container) {
    container = document.createElement('div');
    container.id = 'ap-toast-container';
    container.className = 'toast-container';
    document.body.appendChild(container);
  }

  const toast = document.createElement('div');
  toast.className = `toast toast-${escapeHtml(type)}`;
  toast.innerHTML = (
    `<span class="toast-message">${escapeHtml(message)}</span>` +
    `<button class="toast-dismiss" aria-label="Dismiss">&times;</button>`
  );

  const dismiss = () => {
    toast.classList.add('toast-fadeout');
    setTimeout(() => toast.remove(), 300);
  };

  toast.querySelector('.toast-dismiss').addEventListener('click', dismiss);
  container.appendChild(toast);

  setTimeout(dismiss, 3000);
}
