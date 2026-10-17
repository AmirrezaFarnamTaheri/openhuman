import { afterEach, describe, expect, it, vi } from 'vitest';

import { createWalkthroughSteps } from '../walkthroughSteps';

afterEach(() => {
  document.body.innerHTML = '';
});

describe('walkthrough navigation', () => {
  it.each([
    ['skills-grid', '/connections?tab=composio'],
    ['skills-channels', '/connections?tab=channels'],
  ])('navigates to the correct Connections destination for %s', async (target, route) => {
    const navigate = vi.fn();
    const element = document.createElement('div');
    element.dataset.walkthrough = target;
    document.body.appendChild(element);

    const step = createWalkthroughSteps(navigate).find(
      candidate => candidate.target === `[data-walkthrough="${target}"]`
    );
    expect(step?.before).toBeDefined();

    await step?.before?.();

    expect(navigate).toHaveBeenCalledWith(route);
  });
});
