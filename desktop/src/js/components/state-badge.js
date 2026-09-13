import { escapeHtml } from '../utils/escape-html.js';

/** State-to-label and color mapping. */
const STATE_MAP = {
  created:       { label: 'Created',       cssClass: 'state-created' },
  specified:     { label: 'Specified',     cssClass: 'state-specified' },
  researched:    { label: 'Researched',    cssClass: 'state-researched' },
  planned:       { label: 'Planned',       cssClass: 'state-planned' },
  implementing:  { label: 'Implementing',   cssClass: 'state-implementing' },
  validated:     { label: 'Validated',     cssClass: 'state-validated' },
  shipped:       { label: 'Shipped',       cssClass: 'state-shipped' },
  retrospected:  { label: 'Retrospected',  cssClass: 'state-retrospected' },
};

/** Valid state transitions. */
const TRANSITIONS = {
  created:      ['specified'],
  specified:    ['researched'],
  researched:   ['planned'],
  planned:      ['implementing'],
  implementing: ['validated'],
  validated:    ['shipped'],
  shipped:      ['retrospected'],
  retrospected: [],
};

/**
 * Render a colored state badge.
 * @param {string} state
 * @returns {string} HTML string
 */
export function renderStateBadge(state) {
  const info = STATE_MAP[state] || { label: state, cssClass: 'state-unknown' };
  return `<span class="badge ${escapeHtml(info.cssClass)}">${escapeHtml(info.label)}</span>`;
}

/**
 * Get the next valid state for a given state.
 * @param {string} state
 * @returns {string|null}
 */
export function getNextState(state) {
  const next = TRANSITIONS[state];
  if (next && next.length > 0) return next[0];
  return null;
}

/**
 * Get all possible next states.
 * @param {string} state
 * @returns {string[]}
 */
export function getNextStates(state) {
  return TRANSITIONS[state] || [];
}

/**
 * Render the full state machine visualization.
 * @param {string} currentState
 * @returns {string} HTML string
 */
export function renderStateMachine(currentState) {
  const states = Object.keys(STATE_MAP);
  const currentIdx = states.indexOf(currentState);
  let html = '<div class="state-machine">';
  states.forEach((s, i) => {
    const info = STATE_MAP[s];
    const isActive = i === currentIdx;
    const isPast = i < currentIdx;
    const cls = isActive ? 'sm-active' : isPast ? 'sm-past' : 'sm-future';
    html += `<span class="sm-step ${cls}">`;
    html += `<span class="sm-dot"></span>`;
    html += `<span class="sm-label">${escapeHtml(info.label)}</span>`;
    html += `</span>`;
    if (i < states.length - 1) {
      html += `<span class="sm-connector ${i < currentIdx ? 'sm-conn-past' : ''}"></span>`;
    }
  });
  html += '</div>';
  return html;
}
