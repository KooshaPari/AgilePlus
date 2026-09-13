import { escapeHtml } from '../utils/escape-html.js';

/**
 * Show a modal dialog.
 * @param {{ title: string, content: string, onConfirm?: function, onCancel?: function }} opts
 * @returns {{ close: function }} Handle to close the modal programmatically
 */
export function showModal({ title, content, onConfirm, onCancel }) {
  const existing = document.getElementById('ap-modal-overlay');
  if (existing) existing.remove();

  const overlay = document.createElement('div');
  overlay.id = 'ap-modal-overlay';
  overlay.className = 'modal-overlay';
  overlay.innerHTML = (
    `<div class="modal">` +
      `<div class="modal-header">` +
        `<span class="modal-title">${escapeHtml(title)}</span>` +
        `<button class="modal-close" aria-label="Close">&times;</button>` +
      `</div>` +
      `<div class="modal-body">${content}</div>` +
      `<div class="modal-footer">` +
        `<button class="btn btn-secondary modal-cancel-btn">Cancel</button>` +
        `<button class="btn btn-primary modal-confirm-btn">Confirm</button>` +
      `</div>` +
    `</div>`
  );

  document.body.appendChild(overlay);

  const close = () => overlay.remove();

  overlay.querySelector('.modal-close').addEventListener('click', () => {
    close();
    if (onCancel) onCancel();
  });
  overlay.querySelector('.modal-cancel-btn').addEventListener('click', () => {
    close();
    if (onCancel) onCancel();
  });
  overlay.querySelector('.modal-confirm-btn').addEventListener('click', () => {
    close();
    if (onConfirm) onConfirm();
  });
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) {
      close();
      if (onCancel) onCancel();
    }
  });

  const firstInput = overlay.querySelector('input, textarea');
  if (firstInput) firstInput.focus();

  return { close };
}
