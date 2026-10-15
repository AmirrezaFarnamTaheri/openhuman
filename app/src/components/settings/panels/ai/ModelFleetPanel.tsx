/** Model Fleet settings: opt-in preset syncing, multi-select editing, and image API routing.
 * All mutations go through the existing Settings -> Rust core persistence path.
 */
import { useEffect, useMemo, useState } from 'react';

import { useT } from '../../../../lib/i18n/I18nContext';
import {
  loadModelFleetStatus,
  type ModelFleetStatus,
  resetModelFleetStatus,
} from '../../../../services/api/aiSettingsApi';
import type { ModelRegistryEntry } from '../../../../utils/tauriCommands/config';
import Button from '../../../ui/Button';
import TextField from '../../../ui/TextField';
import type { AISettings, CloudProvider } from './aiPanelTypes';
import { summarizeFleetHealth } from './fleetHealthInsights';
import { FLEET_MODEL_PRESETS } from './modelFleetCatalog';
import {
  applyModelBulkPatch,
  fleetModelKey,
  syncConfiguredProviderPresets,
  syncModelPresets,
  updateSelectedGroupSettings,
  validModelFleetImageEndpoint,
} from './modelFleetOps';
import { validProdexDestination } from './prodexEndpointPolicy';
import { ProviderAccountFleet } from './ProviderAccountFleet';

interface Props {
  settings: AISettings;
  persist: (settings: AISettings) => Promise<void>;
}

const keyOf = (m: ModelRegistryEntry): string => fleetModelKey(m.provider, m.id);

export function ModelFleetPanel({ settings, persist }: Props) {
  const { t } = useT();
  const [busy, setBusy] = useState(false);
  const [fleetHealth, setFleetHealth] = useState<ModelFleetStatus | null>(null);
  const [healthBusy, setHealthBusy] = useState(false);
  const [healthError, setHealthError] = useState('');
  const healthSummary = useMemo(() => summarizeFleetHealth(fleetHealth), [fleetHealth]);
  useEffect(() => {
    let cancelled = false;
    void loadModelFleetStatus()
      .then(status => {
        if (!cancelled) setFleetHealth(status);
      })
      .catch(() => {
        if (!cancelled) setHealthError('Fleet health is unavailable.');
      });
    return () => {
      cancelled = true;
    };
  }, []);
  const refreshHealth = async () => {
    setHealthBusy(true);
    setHealthError('');
    try {
      setFleetHealth(await loadModelFleetStatus());
    } catch {
      setHealthError('Fleet health is unavailable.');
    } finally {
      setHealthBusy(false);
    }
  };
  const resetHealth = async () => {
    const slugs = [...selectedProviders];
    if (slugs.length === 0) return;
    setHealthBusy(true);
    setHealthError('');
    try {
      for (const slug of slugs) await resetModelFleetStatus(slug);
      setFleetHealth(await loadModelFleetStatus());
    } catch (err) {
      setHealthError(err instanceof Error ? err.message : String(err));
    } finally {
      setHealthBusy(false);
    }
  };
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [selectedProviders, setSelectedProviders] = useState<Set<string>>(() => new Set());
  const [selectedModels, setSelectedModels] = useState<Set<string>>(() => new Set());
  const [group, setGroup] = useState('');
  const [groupImageUrl, setGroupImageUrl] = useState('');
  const [strategy, setStrategy] = useState<'round_robin' | 'least_busy' | 'priority_weighted'>(
    'round_robin'
  );
  const [modelId, setModelId] = useState('');
  const [modelImageUrl, setModelImageUrl] = useState('');
  const [search, setSearch] = useState('');
  const [visionAction, setVisionAction] = useState<'keep' | 'on' | 'off'>('keep');
  const [contextAction, setContextAction] = useState('');
  const [confirmDelete, setConfirmDelete] = useState(false);

  const providers = settings.cloudProviders;
  const effectiveGroup = providers.some(p => p.slug === group) ? group : (providers[0]?.slug ?? '');
  const activeProvider = providers.find(p => p.slug === effectiveGroup);
  const providerPresetDelta = useMemo(
    () => syncConfiguredProviderPresets(providers.map(p => ({ ...p, auth_style: p.authStyle }))),
    [providers]
  );
  const modelDelta = useMemo(
    () => syncModelPresets(settings.modelRegistry),
    [settings.modelRegistry]
  );
  const presetNames = useMemo(
    () => new Map(FLEET_MODEL_PRESETS.map(p => [fleetModelKey(p.provider, p.id), p.displayName])),
    []
  );
  const shownModels = useMemo(
    () =>
      settings.modelRegistry.filter(model => {
        const query = search.trim().toLowerCase();
        return (
          !query ||
          (model.provider + '/' + model.id + ' ' + (presetNames.get(keyOf(model)) ?? ''))
            .toLowerCase()
            .includes(query)
        );
      }),
    [settings.modelRegistry, search, presetNames]
  );

  const run = async (operation: () => Promise<void>, successMessage: string) => {
    if (busy) return;
    setBusy(true);
    setError('');
    setNotice('');
    try {
      await operation();
      setNotice(successMessage);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };
  const toggle = (state: Set<string>, change: (value: Set<string>) => void, value: string) => {
    const copy = new Set(state);
    if (copy.has(value)) copy.delete(value);
    else copy.add(value);
    change(copy);
  };

  const syncModels = () =>
    void run(
      async () => {
        await persist({ ...settings, modelRegistry: modelDelta.models });
        setSelectedModels(new Set());
      },
      t('settings.ai.fleet.synced', 'Model presets synchronized without changing your credentials.')
    );

  const syncProviders = () =>
    void run(
      async () => {
        const next: CloudProvider[] = providerPresetDelta.providers.map(p => {
          const { auth_style, ...existing } = p;
          return { ...existing, authStyle: auth_style };
        });
        await persist({ ...settings, cloudProviders: next });
      },
      t('settings.ai.fleet.providerSynced', 'Configured provider presets synchronized.')
    );

  const applyProviderBulk = (clear = false) =>
    void run(
      async () => {
        if (!selectedProviders.size) throw new Error('Select one or more providers.');
        if (!clear && !validModelFleetImageEndpoint(groupImageUrl)) {
          throw new Error('Image endpoints require HTTPS or loopback HTTP.');
        }
        if (
          !clear &&
          selectedProviders.has('prodex') &&
          groupImageUrl.trim() &&
          !validProdexDestination(groupImageUrl.trim())
        ) {
          throw new Error(
            'Prodex image endpoints require HTTPS or loopback HTTP without URL credentials.'
          );
        }
        const next = updateSelectedGroupSettings(
          providers,
          selectedProviders,
          clear
            ? { imageEndpoint: '' }
            : {
                ...(groupImageUrl.trim() ? { imageEndpoint: groupImageUrl.trim() } : {}),
                rotationStrategy: strategy,
              }
        );
        await persist({ ...settings, cloudProviders: next });
        setSelectedProviders(new Set());
      },
      t('settings.ai.fleet.providerUpdated', 'Selected provider groups updated.')
    );

  const saveModelImageEndpoint = (remove = false) =>
    void run(
      async () => {
        if (!activeProvider) throw new Error('Choose a provider group.');
        const id = modelId.trim();
        if (!id || id.length > 200 || /\p{Cc}/u.test(id)) {
          throw new Error('Enter a valid model id (up to 200 characters).');
        }
        if (!remove && (!modelImageUrl.trim() || !validModelFleetImageEndpoint(modelImageUrl))) {
          throw new Error('Enter an image endpoint using HTTPS or loopback HTTP.');
        }
        if (
          !remove &&
          effectiveGroup === 'prodex' &&
          !validProdexDestination(modelImageUrl.trim())
        ) {
          throw new Error(
            'Prodex model image endpoints require HTTPS or loopback HTTP without URL credentials.'
          );
        }
        const next = providers.map(p => {
          if (p.slug !== effectiveGroup) return p;
          const modelImageEndpoints = { ...(p.modelImageEndpoints ?? {}) };
          if (remove) delete modelImageEndpoints[id];
          else modelImageEndpoints[id] = modelImageUrl.trim();
          return { ...p, modelImageEndpoints };
        });
        await persist({ ...settings, cloudProviders: next });
        setModelImageUrl('');
        setModelId('');
      },
      t('settings.ai.fleet.imageRouteUpdated', 'Keyed image model endpoint saved.')
    );

  const applyModels = (remove = false) =>
    void run(
      async () => {
        if (!selectedModels.size) throw new Error('Select one or more models.');
        let models: ModelRegistryEntry[];
        if (remove) {
          models = settings.modelRegistry.filter(m => !selectedModels.has(keyOf(m)));
        } else {
          const patch: Partial<Pick<ModelRegistryEntry, 'vision' | 'context_window'>> = {};
          if (visionAction !== 'keep') patch.vision = visionAction === 'on';
          if (contextAction.trim()) {
            const count = Number(contextAction.trim());
            if (!Number.isInteger(count) || count < 0 || count > 100_000_000) {
              throw new Error('Context window must be an integer between 0 and 100,000,000.');
            }
            patch.context_window = count;
          }
          if (!Object.keys(patch).length) throw new Error('Choose at least one field to update.');
          models = applyModelBulkPatch(settings.modelRegistry, selectedModels, patch);
        }
        await persist({ ...settings, modelRegistry: models });
        setSelectedModels(new Set());
        setConfirmDelete(false);
      },
      t('settings.ai.fleet.modelsUpdated', 'Selected model presets updated.')
    );

  return (
    <section
      className="space-y-4 rounded-xl border border-line bg-surface p-4"
      data-testid="model-fleet-panel">
      <div className="space-y-1">
        <h3 className="text-sm font-semibold text-content">
          {t('settings.ai.fleet.title', 'Model Fleet')}
        </h3>
        <p className="text-xs text-content-muted">
          {t(
            'settings.ai.fleet.description',
            'Manage provider groups, image-specific API addresses, grouped API keys and model presets. Changes do not switch any workload route.'
          )}
        </p>
      </div>
      {error && (
        <p role="alert" className="text-xs text-coral-600">
          {error}
        </p>
      )}
      {notice && (
        <p role="status" className="text-xs text-content-secondary">
          {notice}
        </p>
      )}

      <div className="space-y-2 border-t border-line-subtle pt-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h4 className="text-sm font-medium text-content">
            {t('settings.ai.fleet.health', 'Credential fleet health')}
          </h4>
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              variant="secondary"
              size="xs"
              disabled={healthBusy}
              onClick={() => void refreshHealth()}>
              {t('settings.ai.fleet.refreshHealth', 'Refresh health')}
            </Button>
            <Button
              type="button"
              variant="secondary"
              size="xs"
              disabled={healthBusy || selectedProviders.size === 0}
              onClick={() => void resetHealth()}
              data-testid="model-fleet-reset-health">
              {t('settings.ai.fleet.resetHealth', 'Reset selected key health')}
            </Button>
          </div>
        </div>
        {healthError && (
          <p className="text-xs text-coral-600" role="alert">
            {healthError}
          </p>
        )}

        <div
          className="rounded-lg border border-line-subtle p-2 text-xs text-content-secondary"
          role="status"
          aria-live="polite"
          data-testid="model-fleet-health-summary">
          {healthSummary.status === 'unobserved' ? (
            <span>
              {t(
                'settings.ai.fleet.noSamples',
                'No key activity observed yet; account quota is unknown.'
              )}
            </span>
          ) : (
            <div className="space-y-1">
              <span className="font-medium text-content">
                {t(
                  'settings.ai.fleet.keyHealthSummary',
                  'Observed key routing health (not account quota)'
                )}
              </span>
              <p>
                {healthSummary.healthy} healthy, {healthSummary.cooling} cooling,{' '}
                {healthSummary.evicted} evicted, {healthSummary.activeLeases} active leases across{' '}
                {healthSummary.modelCount} models.
              </p>
              {healthSummary.observedSuccessRate !== null && (
                <p>
                  {t('settings.ai.fleet.observedSuccesses', 'Observed model-key attempts')}:{' '}
                  {healthSummary.successCount} succeeded, {healthSummary.failureCount} failed (
                  {healthSummary.observedSuccessRate}% successful).{' '}
                  {t('settings.ai.fleet.notQuota', 'These are local outcomes, not provider quota.')}
                </p>
              )}
              {healthSummary.nextRecoveryMs !== null && (
                <p>
                  {t('settings.ai.fleet.earliestCooldown', 'Earliest reported cooldown')}:{' '}
                  {Math.ceil(healthSummary.nextRecoveryMs / 1000)}s
                </p>
              )}
            </div>
          )}
        </div>
        <div className="max-h-40 space-y-2 overflow-y-auto text-xs">
          {fleetHealth?.providers.flatMap(provider =>
            provider.models.map(model => {
              const healthy = model.keys.filter(k => k.status === 'healthy').length;
              const cooling = model.keys.filter(k => k.status === 'cooldown').length;
              const evicted = model.keys.filter(k => k.status === 'evicted').length;
              return (
                <div
                  key={provider.slug + ':' + model.model}
                  className="rounded-lg border border-line-subtle px-2 py-1.5 text-content-secondary">
                  <span className="font-medium text-content">
                    {provider.slug}/{model.model}
                  </span>
                  <span className="ml-2">
                    {healthy} healthy, {cooling} cooling down, {evicted} evicted
                  </span>
                  <span className="ml-2">
                    {model.keys.reduce((sum, k) => sum + k.active_leases, 0)} active leases
                  </span>
                </div>
              );
            })
          )}
          {!fleetHealth?.providers.some(p => p.models.length > 0) && (
            <p className="text-content-muted">No inference key usage recorded in this session.</p>
          )}
        </div>
      </div>
      <div className="space-y-2 border-t border-line-subtle pt-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h4 className="text-sm font-medium text-content">
            {t('settings.ai.fleet.groups', 'Provider groups')}
          </h4>
          <Button
            type="button"
            variant="secondary"
            size="xs"
            disabled={busy || providerPresetDelta.updated === 0}
            onClick={syncProviders}
            data-testid="model-fleet-sync-providers">
            {t('settings.ai.fleet.syncProviders', 'Sync configured provider presets')}
            {providerPresetDelta.updated > 0 ? ' (' + providerPresetDelta.updated + ')' : ''}
          </Button>
        </div>
        <p className="text-xs text-content-muted">
          {t(
            'settings.ai.fleet.groupSyncHint',
            'Sync updates official base URLs and protocols; your own labels, credentials and image overrides remain unchanged.'
          )}
        </p>
        <div className="max-h-44 space-y-1 overflow-y-auto rounded-lg border border-line-subtle p-2">
          {providers.map(p => (
            <label
              key={p.slug}
              className="flex min-w-0 cursor-pointer items-center gap-2 rounded p-1 text-xs text-content">
              <input
                type="checkbox"
                checked={selectedProviders.has(p.slug)}
                onChange={() => toggle(selectedProviders, setSelectedProviders, p.slug)}
                aria-label={'Select ' + p.label}
                disabled={busy}
              />
              <span className="min-w-0 truncate">
                {p.label} <span className="text-content-muted">({p.slug})</span>
              </span>
            </label>
          ))}
          {providers.length === 0 && (
            <p className="text-xs text-content-muted">
              {t('settings.ai.fleet.noProviders', 'Connect a provider to manage its model fleet.')}
            </p>
          )}
        </div>
        {providers.length > 0 && (
          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              variant="tertiary"
              size="xs"
              onClick={() => setSelectedProviders(new Set(providers.map(p => p.slug)))}>
              {t('common.selectAll', 'Select all')}
            </Button>
            <Button
              type="button"
              variant="tertiary"
              size="xs"
              onClick={() => setSelectedProviders(new Set())}>
              {t('common.clear', 'Clear selection')}
            </Button>
            <span className="text-xs text-content-muted">{selectedProviders.size} selected</span>
          </div>
        )}
        <div className="grid gap-2 sm:grid-cols-2">
          <label className="space-y-1 text-xs text-content-secondary">
            <span>
              {t('settings.ai.fleet.imageBase', 'Image/vision API base URL for selected groups')}
            </span>
            <TextField
              value={groupImageUrl}
              onChange={e => setGroupImageUrl(e.target.value)}
              placeholder="https://vision.example.com/v1"
              aria-label="Group image API base URL"
            />
          </label>
          <label className="space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.rotation', 'API key routing')}</span>
            <select
              value={strategy}
              onChange={e => setStrategy(e.target.value as typeof strategy)}
              className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm text-content"
              aria-label="Multi-token selection strategy">
              <option value="round_robin">Round robin</option>
              <option value="least_busy">Least busy</option>
              <option value="priority_weighted">Priority weighted</option>
            </select>
          </label>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="primary"
            size="xs"
            disabled={busy || !selectedProviders.size}
            onClick={() => applyProviderBulk(false)}
            data-testid="model-fleet-bulk-providers">
            {t('settings.ai.fleet.applyGroups', 'Apply to selected groups')}
          </Button>
          <Button
            type="button"
            variant="secondary"
            size="xs"
            disabled={busy || !selectedProviders.size}
            onClick={() => applyProviderBulk(true)}>
            {t('settings.ai.fleet.clearImageUrl', 'Clear group image URL')}
          </Button>
        </div>
      </div>

      <ProviderAccountFleet providers={providers} />

      <div className="space-y-2 border-t border-line-subtle pt-3">
        <h4 className="text-sm font-medium text-content">
          {t('settings.ai.fleet.modelImageOverrides', 'Keyed image model endpoints')}
        </h4>
        <p className="text-xs text-content-muted">
          {t(
            'settings.ai.fleet.imageOverrideDesc',
            'Choose a provider group by its slug; each model id may have its own image URL, overriding the group image URL.'
          )}
        </p>
        <div className="grid gap-2 sm:grid-cols-3">
          <label className="space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.providerGroup', 'Provider group')}</span>
            <select
              value={effectiveGroup}
              onChange={e => {
                setGroup(e.target.value);
                setModelId('');
                setModelImageUrl('');
              }}
              className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-xs text-content"
              aria-label="Image endpoint provider group">
              {providers.map(p => (
                <option key={p.slug} value={p.slug}>
                  {p.label}
                </option>
              ))}
            </select>
          </label>
          <label className="space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.modelId', 'Model id / key')}</span>
            <TextField
              value={modelId}
              onChange={e => setModelId(e.target.value)}
              aria-label="Image endpoint model id"
              placeholder="vendor/model-vision"
            />
          </label>
          <label className="space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.imageUrl', 'Image API URL')}</span>
            <TextField
              value={modelImageUrl}
              onChange={e => setModelImageUrl(e.target.value)}
              aria-label="Model image API base URL"
              placeholder="https://vision.example.com/v1"
            />
          </label>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="primary"
            size="xs"
            disabled={busy || !activeProvider}
            onClick={() => saveModelImageEndpoint(false)}
            data-testid="model-fleet-save-image-endpoint">
            {t('common.save', 'Save mapping')}
          </Button>
          <Button
            type="button"
            variant="secondary"
            size="xs"
            disabled={busy || !activeProvider || !modelId.trim()}
            onClick={() => saveModelImageEndpoint(true)}>
            {t('common.remove', 'Remove mapping')}
          </Button>
        </div>
        <div className="max-h-28 space-y-1 overflow-y-auto">
          {Object.entries(activeProvider?.modelImageEndpoints ?? {}).map(([id, url]) => (
            <button
              type="button"
              key={id}
              className="block max-w-full truncate text-left font-mono text-xs text-content-secondary underline"
              title={url}
              onClick={() => {
                setModelId(id);
                setModelImageUrl(url);
              }}>
              {id}: {url}
            </button>
          ))}
        </div>
      </div>

      <div className="space-y-2 border-t border-line-subtle pt-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h4 className="text-sm font-medium text-content">
            {t('settings.ai.fleet.models', 'Model presets')} ({settings.modelRegistry.length})
          </h4>
          <Button
            type="button"
            variant="secondary"
            size="xs"
            disabled={busy || (modelDelta.added === 0 && modelDelta.updated === 0)}
            onClick={syncModels}
            data-testid="model-fleet-sync-models">
            {t('settings.ai.fleet.syncModels', 'Sync model presets')}
            {' (' + modelDelta.added + ' new, ' + modelDelta.updated + ' updated)'}
          </Button>
        </div>
        <p className="text-xs text-content-muted">
          {t(
            'settings.ai.fleet.presetHint',
            'Imports 127 Penguin presets and refreshes known pricing, context and vision flags. Custom models are preserved; routes and credentials are untouched.'
          )}
        </p>
        <TextField
          value={search}
          onChange={e => setSearch(e.target.value)}
          placeholder={t('settings.ai.fleet.search', 'Search model or provider')}
          aria-label="Filter model presets"
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="tertiary"
            size="xs"
            onClick={() => setSelectedModels(new Set(shownModels.map(keyOf)))}
            disabled={!shownModels.length}>
            {t('common.selectAll', 'Select visible')}
          </Button>
          <Button
            type="button"
            variant="tertiary"
            size="xs"
            onClick={() => setSelectedModels(new Set())}
            disabled={!selectedModels.size}>
            {t('common.clear', 'Clear selection')}
          </Button>
          <span className="text-xs text-content-muted">{selectedModels.size} selected</span>
        </div>
        <div className="max-h-52 overflow-y-auto rounded-lg border border-line-subtle">
          {shownModels.map(m => (
            <label
              key={keyOf(m)}
              className="flex cursor-pointer items-center gap-2 border-b border-line-subtle px-2 py-1.5 text-xs text-content">
              <input
                type="checkbox"
                checked={selectedModels.has(keyOf(m))}
                disabled={busy}
                onChange={() => toggle(selectedModels, setSelectedModels, keyOf(m))}
                aria-label={'Select model ' + m.provider + ' ' + m.id}
              />
              <span className="min-w-0 flex-1 truncate" title={m.id}>
                {presetNames.get(keyOf(m)) ?? m.id}
                <span className="ml-1 font-mono text-content-muted">
                  ({m.provider}/{m.id})
                </span>
              </span>
              {m.vision && <span className="text-content-muted">vision</span>}
            </label>
          ))}
          {shownModels.length === 0 && (
            <p className="p-3 text-xs text-content-muted">
              {t(
                'settings.ai.fleet.noModels',
                'No matching models. Sync presets to populate the catalog.'
              )}
            </p>
          )}
        </div>
        <div className="grid gap-2 sm:grid-cols-2">
          <label className="space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.vision', 'Bulk vision support')}</span>
            <select
              value={visionAction}
              onChange={e => setVisionAction(e.target.value as typeof visionAction)}
              className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm text-content">
              <option value="keep">Keep unchanged</option>
              <option value="on">Enabled</option>
              <option value="off">Disabled</option>
            </select>
          </label>
          <label className="space-y-1 text-xs text-content-secondary">
            <span>{t('settings.ai.fleet.context', 'Bulk context window (0 = unknown)')}</span>
            <TextField
              inputMode="numeric"
              value={contextAction}
              onChange={e => setContextAction(e.target.value)}
              placeholder="Leave blank to keep"
              aria-label="Bulk context tokens"
            />
          </label>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="primary"
            size="xs"
            disabled={busy || !selectedModels.size}
            onClick={() => applyModels(false)}
            data-testid="model-fleet-bulk-models">
            {t('settings.ai.fleet.applyModels', 'Apply to selected models')}
          </Button>
          {confirmDelete ? (
            <Button
              type="button"
              variant="secondary"
              size="xs"
              disabled={busy}
              onClick={() => applyModels(true)}>
              {t('settings.ai.fleet.confirmDelete', 'Confirm remove selected')}
            </Button>
          ) : (
            <Button
              type="button"
              variant="secondary"
              size="xs"
              disabled={busy || !selectedModels.size}
              onClick={() => setConfirmDelete(true)}>
              {t('settings.ai.fleet.removeSelected', 'Remove selected models')}
            </Button>
          )}
        </div>
      </div>
    </section>
  );
}
