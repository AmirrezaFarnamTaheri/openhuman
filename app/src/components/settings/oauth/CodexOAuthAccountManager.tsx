/** Explicit selection of authorized Codex OAuth accounts. The native vault
 * is the only credential owner; the renderer receives sanitized profile DTOs.
 */
import { useEffect, useState } from 'react';

import { useT } from '../../../lib/i18n/I18nContext';
import Button from '../../ui/Button';
import {
  authActivateProviderCredentials,
  authListProviderCredentials,
  authRemoveProviderCredentials,
  type AuthProfileSummary,
} from '../../../utils/tauriCommands/auth';

export function CodexOAuthAccountManager({ onActivated }: {
  onActivated: () => Promise<void> | void;
}) {
  const { t } = useT();
  const [accounts, setAccounts] = useState<AuthProfileSummary[]>([]);
  const [refresh, setRefresh] = useState(0);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [pendingRemove, setPendingRemove] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError('');
    void authListProviderCredentials('provider:openai')
      .then(response => {
        if (!cancelled) setAccounts((response.result ?? []).filter(item => item.kind === 'oauth'));
      })
      .catch(() => {
        if (!cancelled) setError(t('settings.ai.codexAccountsLoadError', 'Could not load ChatGPT accounts.'));
      })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [refresh, t]);

  const activate = async (profile: string) => {
    if (busy) return;
    setBusy(true);
    setError('');
    setPendingRemove(null);
    try {
      await authActivateProviderCredentials({ provider: 'provider:openai', profile });
      setRefresh(value => value + 1);
      await onActivated();
    } catch {
      setError(t('settings.ai.codexAccountsActivateError', 'Could not select the ChatGPT account.'));
    } finally { setBusy(false); }
  };

  const remove = async (profile: string) => {
    if (busy) return;
    if (pendingRemove !== profile) { setPendingRemove(profile); return; }
    setBusy(true);
    setError('');
    try {
      await authRemoveProviderCredentials({ provider: 'provider:openai', profile });
      setPendingRemove(null);
      setRefresh(value => value + 1);
      await onActivated();
    } catch {
      setError(t('settings.ai.codexAccountsRemoveError', 'Could not remove the ChatGPT account.'));
    } finally { setBusy(false); }
  };

  return (
    <section className="space-y-2 rounded-lg border border-line-subtle bg-surface p-3 text-xs"
      aria-label={t('settings.ai.codexAccountsTitle', 'Saved ChatGPT accounts')}
      data-testid="codex-oauth-account-list">
      <p className="text-content-muted">
        {t('settings.ai.codexAccountsDescription',
          'Sign-ins are stored separately in encrypted profiles. Select the account used for new Codex requests.')}
      </p>
      {error && <p role="alert" className="text-coral-600">{error}</p>}
      {loading && <p role="status" className="text-content-muted">
        {t('settings.ai.codexAccountsLoading', 'Loading accounts…')}
      </p>}
      {!loading && accounts.map((account, index) => (
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-line-subtle p-2"
          key={account.id} data-testid="codex-oauth-account-row">
          <span className="min-w-0 font-medium text-content">
            {t('settings.ai.codexAccountsLabel', 'ChatGPT account')} {index + 1}
            {account.isActive && (
              <span className="ml-2 text-content-secondary" aria-label="Selected account">
                {t('settings.ai.codexAccountsSelected', 'Selected')}
              </span>
            )}
          </span>
          <div className="flex flex-wrap gap-2">
            {!account.isActive && (
              <Button type="button" variant="tertiary" size="xs" disabled={busy}
                onClick={() => void activate(account.profileName)}>
                {t('settings.ai.codexAccountsUse', 'Use account')}
              </Button>
            )}
            <Button type="button" variant="secondary" size="xs" disabled={busy}
              onClick={() => void remove(account.profileName)}>
              {pendingRemove === account.profileName
                ? t('settings.ai.codexAccountsConfirmRemove', 'Confirm remove')
                : t('settings.ai.codexAccountsRemove', 'Remove')}
            </Button>
          </div>
        </div>
      ))}
      {!loading && !error && accounts.length === 0 && (
        <p className="text-content-muted">{t('settings.ai.codexAccountsEmpty', 'No saved ChatGPT accounts.')}</p>
      )}
    </section>
  );
}
