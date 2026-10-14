import { fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../../../../test/test-utils';
import {
  authActivateProviderCredentials,
  authListProviderCredentials,
  authRemoveProviderCredentials,
  authStoreProviderCredentials,
} from '../../../../../utils/tauriCommands/auth';
import type { CloudProvider } from '../aiPanelTypes';
import { ProviderAccountFleet } from '../ProviderAccountFleet';

vi.mock('../../../../../utils/tauriCommands/auth', () => ({
  authListProviderCredentials: vi.fn(),
  authActivateProviderCredentials: vi.fn(),
  authRemoveProviderCredentials: vi.fn(),
  authStoreProviderCredentials: vi.fn(),
}));

const google: CloudProvider = {
  id: 'google-1',
  slug: 'google',
  label: 'Google Gemini',
  endpoint: 'https://generativelanguage.googleapis.com/v1beta/openai',
  authStyle: 'bearer',
  maskedKey: '',
};

describe('ProviderAccountFleet', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(authListProviderCredentials).mockResolvedValue({
      result: [
        {
          id: 'provider:google:work',
          provider: 'provider:google',
          profileName: 'work',
          kind: 'token',
          accountId: null,
          workspaceId: null,
          metadataKeys: [],
          updatedAt: '2026-10-14T00:00:00Z',
          hasToken: true,
          hasTokenSet: false,
          isActive: true,
        },
      ],
    } as never);
    vi.mocked(authStoreProviderCredentials).mockResolvedValue({ result: {} } as never);
    vi.mocked(authActivateProviderCredentials).mockResolvedValue({ result: {} } as never);
    vi.mocked(authRemoveProviderCredentials).mockResolvedValue({
      result: { removed: true },
    } as never);
  });

  it('renders as a closed native disclosure and loads sanitized camelCase account summaries', async () => {
    renderWithProviders(<ProviderAccountFleet providers={[google]} />);
    expect(screen.getByTestId('model-fleet-accounts-toggle')).toBeInTheDocument();
    expect(screen.queryByTestId('model-fleet-account-list')).not.toBeInTheDocument();
    fireEvent.click(screen.getByTestId('model-fleet-accounts-toggle'));
    expect(await screen.findByText('work')).toBeInTheDocument();
    expect(screen.getByLabelText('Preferred account')).toBeInTheDocument();
    expect(authListProviderCredentials).toHaveBeenCalledWith('provider:google');
  });

  it('adds a new key as an inactive account without rendering or persisting it in settings', async () => {
    renderWithProviders(<ProviderAccountFleet providers={[google]} />);
    fireEvent.click(screen.getByTestId('model-fleet-accounts-toggle'));
    await screen.findByText('work');
    fireEvent.change(screen.getByLabelText('New account label'), { target: { value: 'personal' } });
    fireEvent.change(screen.getByLabelText('New account API key'), {
      target: { value: 'secret-test' },
    });
    fireEvent.click(screen.getByText('Add account'));
    await waitFor(() =>
      expect(authStoreProviderCredentials).toHaveBeenCalledWith({
        provider: 'provider:google',
        profile: 'personal',
        token: 'secret-test',
        setActive: false,
      })
    );
    expect(screen.queryByText('secret-test')).not.toBeInTheDocument();
  });

  it('does not show account management for delegated Prodex only', () => {
    renderWithProviders(<ProviderAccountFleet providers={[{ ...google, slug: 'prodex' }]} />);
    expect(screen.queryByTestId('model-fleet-accounts-toggle')).not.toBeInTheDocument();
  });
});
