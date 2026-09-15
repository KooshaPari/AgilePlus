/**
 * Tauri invoke() wrapper.
 * All backend calls go through this module.
 */

/**
 * @param {string} command - Tauri command name
 * @param {object} [args={}] - Command arguments
 * @returns {Promise<any>} Command result
 */
export async function invoke(command, args = {}) {
  if (window.__TAURI__ && window.__TAURI__.core) {
    return window.__TAURI__.core.invoke(command, args);
  }
  throw new Error(
    `Tauri IPC not available. Cannot invoke '${command}'. ` +
    'This usually means the app is not running inside a Tauri webview.'
  );
}
