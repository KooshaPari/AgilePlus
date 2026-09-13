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
  return window.__TAURI__.core.invoke(command, args);
}
