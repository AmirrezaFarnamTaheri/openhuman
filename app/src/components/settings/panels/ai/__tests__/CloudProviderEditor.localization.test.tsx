import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { CloudProviderEditor } from '../CloudProviderEditor';

vi.mock('../../../../../lib/i18n/I18nContext', () => ({
  useT: () => ({ t: (key: string) => `translated:${key}` }),
}));

describe('CloudProviderEditor localization', () => {
  it('routes model transport, rotation and tool codec controls through useT', async () => {
    render(
      <CloudProviderEditor
        initial={null}
        existingSlugs={[]}
        onClose={() => undefined}
        onSubmit={() => undefined}
        onClearKey={() => undefined}
      />
    );

    await screen.findByRole('dialog');
    expect(screen.getByLabelText('translated:settings.ai.modelTransportOverridesLabel')).toBeTruthy();
    expect(screen.getByLabelText('translated:settings.ai.rotationStrategyLabel')).toBeTruthy();
    expect(screen.getByLabelText('translated:settings.ai.toolCodecLabel')).toBeTruthy();
    expect(screen.getByRole('option', { name: 'translated:settings.ai.rotationLeastBusy' })).toBeTruthy();
    expect(screen.getByRole('option', { name: 'translated:settings.ai.toolCodecTaggedJson' })).toBeTruthy();
    expect(screen.getByText('translated:settings.ai.modelTransportOverridesHelp')).toBeTruthy();
    expect(screen.getByText('translated:settings.ai.toolCodecHelp')).toBeTruthy();
  });
});
