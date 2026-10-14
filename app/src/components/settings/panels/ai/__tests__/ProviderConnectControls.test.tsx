import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { I18nProvider } from '../../../../../lib/i18n/I18nContext';
import { renderWithProviders } from '../../../../../test/test-utils';
import { ProviderKeyDialog } from '../ProviderConnectControls';

function renderKeyDialog(onSubmit = vi.fn().mockResolvedValue(undefined)) {
  renderWithProviders(
    <I18nProvider>
      <ProviderKeyDialog
        slug="openai"
        label="OpenAI"
        isLocalRuntime={false}
        onCancel={vi.fn()}
        onSubmit={onSubmit}
      />
    </I18nProvider>
  );
  return onSubmit;
}

describe('ProviderKeyDialog multi-token security', () => {
  it('uses native password inputs for every token until explicitly revealed', async () => {
    renderKeyDialog();
    fireEvent.click(screen.getByRole('checkbox', { name: /use multiple api keys/i }));
    const entries = screen.getByRole('group', { name: 'API key entries' });
    expect(within(entries).getAllByLabelText(/API key [0-9]+/i)).toHaveLength(1);
    expect(within(entries).getByLabelText('API key 1')).toHaveAttribute('type', 'password');
    fireEvent.click(screen.getByRole('button', { name: /add another api key/i }));
    expect(within(entries).getByLabelText('API key 2')).toHaveAttribute('type', 'password');
    fireEvent.click(screen.getByRole('button', { name: /reveal api keys/i }));
    expect(within(entries).getByLabelText('API key 1')).toHaveAttribute('type', 'text');
    expect(within(entries).getByLabelText('API key 2')).toHaveAttribute('type', 'text');
    fireEvent.click(screen.getByRole('button', { name: /hide api keys/i }));
    expect(within(entries).getByLabelText('API key 1')).toHaveAttribute('type', 'password');
  });

  it('normalizes multiline/pasted credentials and keeps each token independent', async () => {
    const onSubmit = renderKeyDialog();
    fireEvent.click(screen.getByRole('checkbox', { name: /use multiple api keys/i }));
    const entries = screen.getByRole('group', { name: 'API key entries' });
    fireEvent.change(within(entries).getByLabelText('API key 1'), {
      target: { value: 'secret-a' },
    });
    fireEvent.click(screen.getByRole('button', { name: /add another api key/i }));
    fireEvent.change(within(entries).getByLabelText('API key 2'), {
      target: { value: 'secret-b' },
    });
    fireEvent.click(screen.getByRole('button', { name: /add another api key/i }));
    fireEvent.change(within(entries).getByLabelText('API key 3'), {
      target: { value: 'secret-a' },
    });
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: /^save$/i }));
    });
    await waitFor(() => expect(onSubmit).toHaveBeenCalledWith('secret-a\nsecret-b'));
  });

  it('keeps an occupied credential when additional keys are pasted into that row', async () => {
    const onSubmit = renderKeyDialog();
    fireEvent.click(screen.getByRole('checkbox', { name: /use multiple api keys/i }));
    const entries = screen.getByRole('group', { name: 'API key entries' });
    const keyOne = within(entries).getByLabelText('API key 1') as HTMLInputElement;
    fireEvent.change(keyOne, { target: { value: 'existing-secret' } });
    keyOne.setSelectionRange(4, 4);
    fireEvent.paste(keyOne, {
      clipboardData: { getData: () => 'new-secret-a\nnew-secret-b' },
    });
    expect(within(entries).getByLabelText('API key 1')).toHaveValue('existing-secret');
    expect(within(entries).getByLabelText('API key 2')).toHaveValue('new-secret-a');
    expect(within(entries).getByLabelText('API key 3')).toHaveValue('new-secret-b');
    await act(async () => fireEvent.click(screen.getByRole('button', { name: /^save$/i })));
    await waitFor(() =>
      expect(onSubmit).toHaveBeenCalledWith('existing-secret\nnew-secret-a\nnew-secret-b')
    );
  });

  it('replaces a fully selected credential with a pasted group', () => {
    renderKeyDialog();
    fireEvent.click(screen.getByRole('checkbox', { name: /use multiple api keys/i }));
    const entries = screen.getByRole('group', { name: 'API key entries' });
    const keyOne = within(entries).getByLabelText('API key 1') as HTMLInputElement;
    fireEvent.change(keyOne, { target: { value: 'replace-me' } });
    keyOne.setSelectionRange(0, keyOne.value.length);
    fireEvent.paste(keyOne, { clipboardData: { getData: () => 'fresh-a;fresh-b' } });
    expect(within(entries).getByLabelText('API key 1')).toHaveValue('fresh-a');
    expect(within(entries).getByLabelText('API key 2')).toHaveValue('fresh-b');
  });

  it('never creates a visible plaintext textarea for multi-token entry', () => {
    renderKeyDialog();
    fireEvent.click(screen.getByRole('checkbox', { name: /use multiple api keys/i }));
    expect(screen.queryByRole('textbox', { name: /API key entries/i })).toBeNull();
    expect(screen.getByRole('group', { name: 'API key entries' })).toBeInTheDocument();
  });
});
