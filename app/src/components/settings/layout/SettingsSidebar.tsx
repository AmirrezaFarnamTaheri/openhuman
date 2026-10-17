import { useState } from 'react';

import { useT } from '../../../lib/i18n/I18nContext';
import TwoPaneNav from '../../layout/TwoPaneNav';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';
import {
  entryRoute,
  NAV_GROUP_LABEL_KEY,
  SETTINGS_ROUTE_REGISTRY,
  resolveSidebarId,
  sidebarGroups,
} from '../settingsRouteRegistry';
import { SETTINGS_NAV_ICONS } from './settingsNavIcons';

/**
 * Grouped settings navigation. On wide viewports this is the persistent left
 * pane of the two-pane layout; on narrow viewports it doubles as the
 * /settings index page (the old drill-down home list).
 *
 * Rendered with the shared {@link TwoPaneNav} — the same primitive every other
 * page's sidebar uses, and the same column it is projected into. It used to
 * hand-roll an equivalent row list, which drifted the moment either side was
 * touched: shorter rows (`py-1` vs `py-1.5`), a different group-heading inset,
 * and a grey icon left sitting on the accent fill of a selected row.
 *
 * Search now reuses the full settings route registry and its keyword aliases,
 * restoring access to deep destinations without recreating the old search
 * helper directory. The default view stays the compact grouped sidebar.
 */
const SettingsSidebar = () => {
  const { t } = useT();
  const { currentRoute, navigateToSettings } = useSettingsNavigation();
  const [search, setSearch] = useState('');

  const activeSidebarId = resolveSidebarId(currentRoute);

  // `route` is not carried on the nav item, so keep the id → route map beside
  // the groups and resolve on select.
  const routeById = new Map<string, string>();

  const groups = sidebarGroups().map(group => ({
    label: t(NAV_GROUP_LABEL_KEY[group.group]),
    testId: `settings-sidebar-group-${group.group}`,
    items: group.entries.map(entry => {
      routeById.set(entry.id, entryRoute(entry));
      return {
        value: entry.id,
        label: t(entry.titleKey),
        icon: SETTINGS_NAV_ICONS[entry.id] ?? null,
        highlight: entry.highlight,
        testId: `settings-nav-${entry.id}`,
      };
    }),
  }));

  const term = search.trim().toLocaleLowerCase();
  const matchingRoutes = term
    ? SETTINGS_ROUTE_REGISTRY.filter(entry => !entry.hiddenDeepLink && !entry.devOnly && (
        t(entry.titleKey).toLocaleLowerCase().includes(term) ||
        entry.id.toLocaleLowerCase().includes(term) ||
        entry.searchKeywords?.some(keyword => keyword.toLocaleLowerCase().includes(term))
      ))
    : [];
  const searchGroups = term ? [{
    label: t('nav.settings'),
    items: matchingRoutes.map(entry => ({
      value: entry.id,
      label: t(entry.titleKey),
      testId: `settings-search-${entry.id}`,
    })),
  }] : groups;
  for (const entry of matchingRoutes) routeById.set(entry.id, entryRoute(entry));

  return (
    <TwoPaneNav
      ariaLabel={t('nav.settings')}
      walkthroughId="settings-menu"
      header={
        <label className="block">
          <span className="sr-only">Search settings</span>
          <input type="search" value={search} onChange={event => setSearch(event.target.value)}
            placeholder="Search settings" aria-label="Search settings"
            className="w-full rounded-md border border-border-subtle bg-surface px-3 py-2 text-sm text-content placeholder:text-content-muted"
          />
        </label>
      }
      groups={searchGroups}
      selected={activeSidebarId ?? ''}
      onSelect={id => {
        const route = routeById.get(id);
        if (route) navigateToSettings(route);
      }}
    />
  );
};

export default SettingsSidebar;
