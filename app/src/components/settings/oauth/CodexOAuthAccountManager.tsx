/** Explicit selection of *already authorized* Codex OAuth profiles.
 * No access/refresh token is fetched by this component.
 */
import { useEffect, useState } from 'react';
import {
  authActivateProviderCredentials,
  authListProviderCredentials,
  authRemoveProviderCredentials,
  type AuthProfileSummary,
} from '../../../utils/tauriCommands/auth';
import { callCoreRpc } from '../../../services/coreRpcClient';

export function CodexOAuthAccountManager({ onActivated }: {
  onActivated: () => Promise<void> | void;
}) {
  const [accounts, setAccounts] = useState<AuthProfileSummary[]>([]);
  const [activeId, setActiveId] = useState('');
  const [refresh, setRefresh] = useState(0);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [pendingRemove, setPendingRemove] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void Promise.all([
      authListProviderCredentials('provider:openai'),
      callCoreRpc<{ result: { profile_id?: string } }>({
        method: 'openhuman.inference_openai_oauth_status', params: {},
      }),
    ]).then(([list, status]) => {
      if (cancelled) return;
      setAccounts((list.result ?? []).filter(account => account.kind === 'oauth'));
      setActiveId(status?.result?.profile_id ?? '');
    }).catch(() => { if (!cancelled) setError('Could not load ChatGPT accounts.'); });
    return () => { cancelled = true; };
  }, [refresh]);

  const activate = async (profile: string) => {
    if (busy) return;
    setBusy(true);
    setError('');
    try {
      await authActivateProviderCredentials({ provider: 'provider:openai', profile });
      setRefresh(value => value + 1);
      await onActivated();
    } catch {
      setError('Could not select the ChatGPT account.');
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
      setError('Could not remove the ChatGPT account.');
    } finally { setBusy(false); }
  };

  return (
    <div className="space-y-2 rounded-lg border border-line-subtle p-2 text-xs"
      data-testid="codex-oauth-account-list">
      <p className="text-content-muted">
        Each login is stored separately in OpenHuman's encrypted credential profiles.
        Switching changes which account signs new Codex inference requests.
      </p>
      {error && <p role="alert" className="text-coral-600">{error}</p>}
      {accounts.map((account, index) => (
        <div className="flex flex-wrap items-center justify-between gap-2" key={account.id}>
          <span className="font-medium">
            ChatGPT account {index + 1}
            {activeId === account.id ? ' · selected' : ''}
          </span>
          <div className="flex gap-2">
            <button type="button" disabled={busy || activeId === account.id}
              onClick={() => void activate(account.profile_name)}
              className="underline disabled:opacity-50">Use account</button>
            <button type="button" disabled={busy}
              onClick={() => void remove(account.profile_name)}
              className="underline text-coral-600 disabled:opacity-50">
              {pendingRemove === account.profile_name ? 'Confirm remove' : 'Remove'}
            </button>
          </div>
        </div>
      ))}
      {accounts.length === 0 && <p className="text-content-muted">No stored ChatGPT accounts.</p>}
    </div>
  );
}
