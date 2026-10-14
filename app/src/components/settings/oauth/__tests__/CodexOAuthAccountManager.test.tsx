import { fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../../../test/test-utils';
import {
  authActivateProviderCredentials, authListProviderCredentials, authRemoveProviderCredentials,
} from '../../../../utils/tauriCommands/auth';
import { CodexOAuthAccountManager } from '../CodexOAuthAccountManager';

vi.mock('../../../../utils/tauriCommands/auth', () => ({
  authListProviderCredentials: vi.fn(),
  authActivateProviderCredentials: vi.fn(),
  authRemoveProviderCredentials: vi.fn(),
}));

const account = (profileName: string, isActive: boolean) => ({
  id: 'provider:openai:' + profileName,
  provider: 'provider:openai',
  profileName,
  kind: 'oauth' as const,
  accountId: null,
  workspaceId: null,
  metadataKeys: [],
  updatedAt: '2026-10-14T00:00:00Z',
  hasToken: false,
  hasTokenSet: true,
  isActive,
});

describe('Codex OAuth account selector', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(authListProviderCredentials).mockResolvedValue({
      result: [account('oauth', true), account('oauth-another', false), {...account('api-key', false), kind: 'token'}],
    } as never);
    vi.mocked(authActivateProviderCredentials).mockResolvedValue({ result: account('oauth-another', true) } as never);
    vi.mocked(authRemoveProviderCredentials).mockResolvedValue({ result: { removed: true } } as never);
  });

  it('renders only OAuth accounts with the selected status from the sanitized RPC', async () => {
    renderWithProviders(<CodexOAuthAccountManager onActivated={vi.fn()} />);
    expect(await screen.findAllByTestId('codex-oauth-account-row')).toHaveLength(2);
    expect(screen.getByLabelText('Selected account')).toBeInTheDocument();
    expect(authListProviderCredentials).toHaveBeenCalledWith('provider:openai');
    expect(screen.queryByText(/api-key/i)).not.toBeInTheDocument();
  });

  it('activates a named OAuth profile using the camelCase profile name', async () => {
    const onActivated = vi.fn().mockResolvedValue(undefined);
    renderWithProviders(<CodexOAuthAccountManager onActivated={onActivated} />);
    const rows = await screen.findAllByTestId('codex-oauth-account-row');
    fireEvent.click(rows[1].querySelector('button')!);
    await waitFor(() => expect(authActivateProviderCredentials).toHaveBeenCalledWith({
      provider: 'provider:openai', profile: 'oauth-another',
    }));
    expect(onActivated).toHaveBeenCalledOnce();
  });

  it('requires confirmation to remove exactly one named profile', async () => {
    renderWithProviders(<CodexOAuthAccountManager onActivated={vi.fn()} />);
    const rows = await screen.findAllByTestId('codex-oauth-account-row');
    fireEvent.click(screen.getAllByText('Remove')[0]);
    expect(authRemoveProviderCredentials).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText('Confirm remove'));
    await waitFor(() => expect(authRemoveProviderCredentials).toHaveBeenCalledWith({
      provider: 'provider:openai', profile: 'oauth',
    }));
    expect(rows.length).toBe(2);
  });
});
