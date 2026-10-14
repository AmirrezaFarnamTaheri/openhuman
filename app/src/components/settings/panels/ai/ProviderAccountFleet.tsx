/** Secure, provider-scoped API-key accounts. Native Rust owns storage and
 * selection; this panel handles sanitized summaries and explicit user actions.
 */
import { useEffect, useMemo, useState } from 'react';

import { useT } from '../../../../lib/i18n/I18nContext';
import Button from '../../../ui/Button';
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from '../../../ui/Collapsible';
import TextField from '../../../ui/TextField';
import {
  authActivateProviderCredentials,
  authListProviderCredentials,
  authRemoveProviderCredentials,
  authStoreProviderCredentials,
  type AuthProfileSummary,
} from '../../../../utils/tauriCommands/auth';
import type { CloudProvider } from './aiPanelTypes';
import { accountFleetEligible, validateAccountApiKey, validateNamedProviderAccount } from './providerAccountOps';

export function ProviderAccountFleet({ providers }: { providers: CloudProvider[] }) {
  const { t } = useT();
  const supported = useMemo(
    () => providers.filter(provider => accountFleetEligible(provider.slug, provider.authStyle)),
    [providers]
  );
  const [chosen, setChosen] = useState('');
  const active = supported.some(provider => provider.slug === chosen)
    ? chosen : (supported[0]?.slug ?? '');
  const [label, setLabel] = useState('');
  const [secret, setSecret] = useState('');
  const [accounts, setAccounts] = useState<AuthProfileSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [removeConfirmation, setRemoveConfirmation] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    let cancelled = false;
    // Clear the previous provider immediately; a slow fetch must never make
    // another provider's account name appear under the new provider selector.
    setAccounts([]);
    setNotice('');
    if (!active) { setLoading(false); return; }
    setLoading(true);
    setError('');
    void authListProviderCredentials('provider:' + active)
      .then(response => {
        if (!cancelled) {
          setAccounts((response.result ?? []).filter(row =>
            row.kind === 'token' && row.provider === 'provider:' + active));
        }
      })
      .catch(() => {
        if (!cancelled) setError(t('settings.ai.fleet.accountsLoadError', 'Could not load saved accounts.'));
      })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [active, revision, t]);

  const add = async () => {
    if (busy || !active || loading) return;
    let profile: string;
    let token: string;
    try {
      profile = validateNamedProviderAccount(label);
      token = validateAccountApiKey(secret);
      if (accounts.some(row => row.profileName === profile)) {
        throw new Error(t('settings.ai.fleet.accountsDuplicate', 'An account with this name already exists.'));
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t('settings.ai.fleet.accountsInvalid', 'Invalid account details.'));
      return;
    }
    setBusy(true);
    setError('');
    setNotice('');
    try {
      await authStoreProviderCredentials({
        provider: 'provider:' + active,
        profile,
        token,
        setActive: false,
      });
      setSecret('');
      setLabel('');
      setRevision(value => value + 1);
      setNotice(t('settings.ai.fleet.accountsAdded', 'Account added to secure storage.'));
    } catch {
      setError(t('settings.ai.fleet.accountsSaveError', 'Could not save account to secure storage.'));
    } finally { setBusy(false); }
  };

  const activate = async (profile: string) => {
    if (busy || !active || loading) return;
    setBusy(true);
    setError('');
    setNotice('');
    try {
      await authActivateProviderCredentials({ provider: 'provider:' + active, profile });
      setRevision(value => value + 1);
      setNotice(t('settings.ai.fleet.accountsSelected', 'Preferred account updated for new model requests.'));
    } catch {
      setError(t('settings.ai.fleet.accountsActivateError', 'Could not select provider account.'));
    } finally { setBusy(false); }
  };

  const remove = async (profile: string) => {
    if (busy || !active || loading) return;
    if (removeConfirmation !== profile) {
      setRemoveConfirmation(profile);
      return;
    }
    setBusy(true);
    setError('');
    setNotice('');
    try {
      await authRemoveProviderCredentials({ provider: 'provider:' + active, profile });
      setRemoveConfirmation(null);
      setRevision(value => value + 1);
      setNotice(t('settings.ai.fleet.accountsRemoved', 'Account removed.'));
    } catch {
      setError(t('settings.ai.fleet.accountsRemoveError', 'Could not remove account.'));
    } finally { setBusy(false); }
  };

  if (supported.length === 0) return null;

  return (
    <CollapsibleRoot variant="plain" className="border-t border-line-subtle pt-2"
      data-testid="model-fleet-accounts">
      <CollapsibleTrigger size="sm" data-testid="model-fleet-accounts-toggle"
        className="rounded-lg">
        <span>{t('settings.ai.fleet.accountsTitle', 'Provider accounts')}</span>
        <span className="text-xs font-normal text-content-muted">
          {t('settings.ai.fleet.accountsExpand', 'Manage API-key rotation')}
        </span>
      </CollapsibleTrigger>
      <CollapsibleContent size="sm">
        <div className="space-y-3 pt-2">
          <p className="text-xs text-content-muted">
            {t('settings.ai.fleet.accountsDescription',
              'Store separate API keys for each provider. OpenHuman rotates healthy keys automatically; Codex and Claude login are managed separately.')}
          </p>
          <label className="block space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.accountsProvider', 'Provider')}</span>
            <select aria-label="Account provider" value={active} disabled={busy}
              onChange={event => {
                setChosen(event.target.value);
                setLabel('');
                setSecret('');
                setRemoveConfirmation(null);
              }}
              className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm text-content">
              {supported.map(provider => (
                <option value={provider.slug} key={provider.slug}>{provider.label}</option>
              ))}
            </select>
          </label>
          <div className="grid gap-2 sm:grid-cols-2">
            <label className="space-y-1 text-xs text-content-secondary">
              <span>{t('settings.ai.fleet.accountsLabel', 'Account label')}</span>
              <TextField aria-label="New account label" value={label} autoComplete="off"
                placeholder="work" disabled={busy}
                onChange={event => setLabel(event.target.value)} />
            </label>
            <label className="space-y-1 text-xs text-content-secondary">
              <span>{t('settings.ai.fleet.accountsKey', 'Provider API key')}</span>
              <TextField aria-label="New account API key" type="password" value={secret}
                autoComplete="off" placeholder="One key per account" disabled={busy}
                onChange={event => setSecret(event.target.value)} />
            </label>
          </div>
          <Button type="button" size="xs" variant="primary"
            disabled={busy || loading || !label.trim() || !secret.trim()} onClick={() => void add()}>
            {t('settings.ai.fleet.accountsAdd', 'Add account')}
          </Button>
          {error && <p role="alert" className="text-xs text-coral-600">{error}</p>}
          {notice && <p role="status" className="text-xs text-content-secondary">{notice}</p>}
          {loading && <p role="status" className="text-xs text-content-muted">
            {t('settings.ai.fleet.accountsLoading', 'Loading accounts…')}
          </p>}
          {!loading && (
            <div className="space-y-1" data-testid="model-fleet-account-list">
              {accounts.map(account => (
                <div key={account.id} className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-line-subtle bg-surface p-2 text-xs">
                  <div className="min-w-0">
                    <span className="font-medium text-content">
                      {account.profileName === 'default'
                        ? t('settings.ai.fleet.accountsDefault', 'Default key')
                        : account.profileName}
                    </span>
                    <span className="ml-2 text-content-muted">
                      {account.hasToken
                        ? t('settings.ai.fleet.accountsStored', 'Stored')
                        : t('settings.ai.fleet.accountsMissing', 'No key')}
                    </span>
                    {account.isActive && (
                      <span className="ml-2 text-content-secondary" aria-label="Preferred account">
                        {t('settings.ai.fleet.accountsPreferred', 'Preferred')}
                      </span>
                    )}
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {!account.isActive && (
                      <Button type="button" variant="tertiary" size="xs" disabled={busy}
                        onClick={() => void activate(account.profileName)}>
                        {t('settings.ai.fleet.accountsPrefer', 'Prefer')}
                      </Button>
                    )}
                    {account.profileName !== 'default' && account.profileName !== 'oauth' && (
                      <Button type="button" variant="secondary" size="xs" disabled={busy}
                        onClick={() => void remove(account.profileName)}>
                        {removeConfirmation === account.profileName
                          ? t('settings.ai.fleet.accountsConfirmRemove', 'Confirm remove')
                          : t('settings.ai.fleet.accountsRemove', 'Remove')}
                      </Button>
                    )}
                  </div>
                </div>
              ))}
              {accounts.length === 0 && !error && (
                <p className="text-xs text-content-muted">
                  {t('settings.ai.fleet.accountsEmpty', 'No accounts saved for this provider.')}
                </p>
              )}
            </div>
          )}
        </div>
      </CollapsibleContent>
    </CollapsibleRoot>
  );
}
