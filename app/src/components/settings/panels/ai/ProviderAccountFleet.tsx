/** Named API-key accounts reuse OpenHuman encrypted native credentials.
 * Subscription OAuth and Claude CLI login have separate lifecycles.
 */
import { useEffect, useMemo, useState } from 'react';
import Button from '../../../ui/Button';
import TextField from '../../../ui/TextField';
import {
  authListProviderCredentials, authRemoveProviderCredentials,
  authStoreProviderCredentials, authActivateProviderCredentials, type AuthProfileSummary,
} from '../../../../utils/tauriCommands/auth';
import type { CloudProvider } from './aiPanelTypes';
import { accountFleetEligible, validateAccountApiKey, validateNamedProviderAccount } from './providerAccountOps';

export function ProviderAccountFleet({ providers }: { providers: CloudProvider[] }) {
  const supported = useMemo(() => providers.filter(p => accountFleetEligible(p.slug, p.authStyle)), [providers]);
  const [chosen, setChosen] = useState('');
  const active = supported.some(p => p.slug === chosen) ? chosen : (supported[0]?.slug ?? '');
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
    if (!active) { setAccounts([]); return; }
    setLoading(true);
    setError('');
    void authListProviderCredentials('provider:' + active)
      .then(result => {
        if (!cancelled) setAccounts((result.result ?? []).filter(row => row.kind === 'token'));
      })
      .catch(() => { if (!cancelled) setError('Could not load account status.'); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [active, revision]);

  if (!supported.length) return null;

  const add = async () => {
    if (busy || !active) return;
    let profile: string;
    let token: string;
    try {
      profile = validateNamedProviderAccount(label);
      token = validateAccountApiKey(secret);
      if (accounts.some(row => row.profileName === profile)) {
        throw new Error('Account label already exists.');
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Invalid account details.');
      return;
    }
    setBusy(true);
    setError('');
    try {
      await authStoreProviderCredentials({
        provider: 'provider:' + active, profile, token, setActive: false,
      });
      setSecret('');
      setLabel('');
      setRevision(value => value + 1);
    } catch {
      setError('Could not save account to native credential storage.');
    } finally { setBusy(false); }
  };

  const activate = async (profile: string) => {
    if (busy || !active) return;
    setBusy(true);
    setError('');
    setNotice('');
    try {
      await authActivateProviderCredentials({ provider: 'provider:' + active, profile });
      setNotice('Preferred account updated. New model requests use the selected profile first.');
      setRevision(value => value + 1);
    } catch {
      setError('Could not select provider account.');
    } finally { setBusy(false); }
  };

  const remove = async (profile: string) => {
    if (busy) return;
    if (removeConfirmation !== profile) { setRemoveConfirmation(profile); return; }
    setBusy(true);
    setError('');
    try {
      await authRemoveProviderCredentials({ provider: 'provider:' + active, profile });
      setRemoveConfirmation(null);
      setRevision(value => value + 1);
    } catch {
      setError('Could not remove account.');
    } finally { setBusy(false); }
  };

  return (
    <section className="space-y-2 border-t border-line-subtle pt-3"
      aria-label="Provider API-key accounts" data-testid="model-fleet-accounts">
      <h4 className="text-sm font-medium text-content">Provider API-key accounts</h4>
      <p className="text-xs text-content-muted">
        Add Google Gemini, Anthropic, OpenAI and compatible API-key accounts.
        Native routing rotates healthy keys. Subscription and CLI OAuth use separate sign-ins.
      </p>
      <label className="block space-y-1 text-xs text-content-secondary">
        <span>Provider</span>
        <select aria-label="Account provider" value={active} disabled={busy}
          onChange={event => {
            setChosen(event.target.value);
            setLabel(''); setSecret(''); setRemoveConfirmation(null);
          }}
          className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm text-content">
          {supported.map(p => <option value={p.slug} key={p.slug}>{p.label}</option>)}
        </select>
      </label>
      <div className="grid gap-2 sm:grid-cols-2">
        <label className="space-y-1 text-xs text-content-secondary">
          <span>Account label</span>
          <TextField aria-label="New account label" value={label} autoComplete="off"
            placeholder="work" disabled={busy} onChange={event => setLabel(event.target.value)} />
        </label>
        <label className="space-y-1 text-xs text-content-secondary">
          <span>Provider API key</span>
          <TextField aria-label="New account API key" type="password" value={secret}
            autoComplete="off" placeholder="One key per account" disabled={busy}
            onChange={event => setSecret(event.target.value)} />
        </label>
      </div>
      <Button type="button" size="xs" variant="primary"
        disabled={busy || !label.trim() || !secret.trim()} onClick={() => void add()}>
        Add account to secure vault
      </Button>
      {error && <p role="alert" className="text-xs text-coral-600">{error}</p>}
      {notice && <p role="status" className="text-xs text-content-secondary">{notice}</p>}
      {loading && <p className="text-xs text-content-muted">Loading accounts…</p>}
      <div className="space-y-1">
        {accounts.map(row => (
          <div key={row.id} className="flex items-center justify-between rounded border border-line-subtle p-2 text-xs">
            <span>{row.profileName === 'default' ? 'Default key' : row.profileName}
              <span className="ml-1 text-content-muted">({row.hasToken ? 'stored' : 'missing key'})</span>
            </span>
            <div className="flex gap-2">
              {row.isActive ? (
                <span aria-label="Preferred account" className="text-xs font-medium text-content-secondary">
                  Preferred
                </span>
              ) : (
                <Button type="button" variant="tertiary" size="xs" disabled={busy}
                  onClick={() => void activate(row.profileName)}>Prefer</Button>
              )}
              {row.profileName !== 'default' && row.profileName !== 'oauth' && (
                <Button type="button" variant="secondary" size="xs" disabled={busy}
                  onClick={() => void remove(row.profileName)}>
                  {removeConfirmation === row.profileName ? 'Confirm remove' : 'Remove'}
                </Button>
              )}
            </div>
          </div>
        ))}
        {!loading && accounts.length === 0 && <p className="text-xs text-content-muted">
          No named accounts. Existing provider connections are unchanged.
        </p>}
      </div>
    </section>
  );
}
