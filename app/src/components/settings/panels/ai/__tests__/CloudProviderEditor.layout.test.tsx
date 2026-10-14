import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { CloudProviderEditor } from '../CloudProviderEditor';

vi.mock('../../../../../lib/i18n/I18nContext', () => ({
  useT: () => ({ t: (key: string) => key }),
}));

describe('CloudProviderEditor layout', () => {
  it('keeps Save and Cancel accessible while advanced fields scroll within viewport', async () => {
    render(
      <CloudProviderEditor
        initial={null}
        existingSlugs={[]}
        onClose={() => undefined}
        onSubmit={() => undefined}
        onClearKey={() => undefined}
      />
    );

    const dialog = await screen.findByRole('dialog');
    expect(dialog.className).toContain('max-h-[calc(100dvh-2rem)]');
    expect(dialog.className).toContain('flex-col');
    const scrollable = dialog.querySelector('.overflow-y-auto');
    expect(scrollable).toBeTruthy();
    expect(scrollable?.className).toContain('min-h-0');
    expect(scrollable?.contains(screen.getByLabelText(/Per-model protocol overrides/))).toBe(true);
    const buttons = Array.from(dialog.querySelectorAll('button'));
    expect(buttons.some(button => button.textContent?.includes('common.cancel'))).toBe(true);
    expect(buttons.some(button => button.textContent?.includes('settings.ai.addProvider'))).toBe(true);
  });
});
