// @vitest-environment jsdom
import { beforeEach, expect, it, vi } from 'vitest';
import { waitFor } from '@testing-library/react';

const { get } = vi.hoisted(() => ({ get: vi.fn() }));
vi.mock('axios', () => ({ default: { get } }));

beforeEach(() => {
  document.body.innerHTML = '<div id="root"></div>';
  get.mockReset();
});

it('reports the API as disconnected when the static host returns HTML for API paths', async () => {
  get.mockResolvedValue({ data: '<!doctype html><html></html>' });

  await import('./main');

  await waitFor(() => {
    expect(document.querySelector('[role="status"]')?.textContent)
      .toContain('This preview is not connected to the AgilePlus API');
  });
  expect(document.body.textContent).not.toContain('✓ 0 epics · 0 stories');
});
