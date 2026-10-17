import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../../test/test-utils';
import SettingsSidebar from './SettingsSidebar';

const navigateToSettings = vi.fn();

vi.mock('../hooks/useSettingsNavigation', () => ({
  useSettingsNavigation: () => ({ currentRoute: 'privacy', navigateToSettings }),
}));

vi.mock('../settingsRouteRegistry', () => ({
  NAV_GROUP_LABEL_KEY: { general: 'nav.group.general' },
  SETTINGS_ROUTE_REGISTRY: [{ id: 'privacy', titleKey: 'Privacy', section: 'account', searchKeywords: ['security'] }],
  entryRoute: (entry: { id: string }) => entry.id,
  resolveSidebarId: (routeId: string) => routeId,
  sidebarGroups: () => [
    {
      group: 'general',
      entries: [
        { id: 'privacy', titleKey: 'Privacy', route: 'privacy' },
        { id: 'account', titleKey: 'Account', route: 'account' },
      ],
    },
  ],
}));

vi.mock('../../../lib/i18n/I18nContext', () => ({ useT: () => ({ t: (key: string) => key }) }));

describe('<SettingsSidebar />', () => {
  it('renders grouped nav rows and marks the active one', () => {
    renderWithProviders(<SettingsSidebar />);

    expect(screen.getByTestId('settings-nav-privacy')).toHaveAttribute('aria-current', 'page');
    expect(screen.getByTestId('settings-nav-account')).not.toHaveAttribute('aria-current');
  });

  it('finds a deep setting by an alternate term', () => {
    renderWithProviders(<SettingsSidebar />);
    fireEvent.change(screen.getByRole('searchbox', { name: /search settings/i }), {
      target: { value: 'security' },
    });
    expect(screen.getByTestId('settings-search-privacy')).toBeInTheDocument();
  });

  it('explains when no settings match instead of leaving an empty sidebar', () => {
    renderWithProviders(<SettingsSidebar />);
    fireEvent.change(screen.getByRole('searchbox', { name: /search settings/i }), {
      target: { value: 'unfindable-one-off-setting' },
    });
    expect(screen.getByTestId('settings-search-empty')).toBeInTheDocument();
  });

  it('opens the first matching setting with Enter and clears its filter', () => {
    renderWithProviders(<SettingsSidebar />);
    const search = screen.getByRole('searchbox', { name: /search settings/i });
    fireEvent.change(search, { target: { value: 'security' } });
    fireEvent.keyDown(search, { key: 'Enter' });
    expect(navigateToSettings).toHaveBeenCalledWith('privacy');
    expect(search).toHaveValue('');
  });

  it('navigates when a nav row is clicked', () => {
    renderWithProviders(<SettingsSidebar />);

    fireEvent.click(screen.getByTestId('settings-nav-account'));
    expect(navigateToSettings).toHaveBeenCalledWith('account');
  });
});
