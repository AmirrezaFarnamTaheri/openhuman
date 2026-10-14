/*
 * Submit handler for the advanced `CloudProviderEditor` modal — live-verifies
 * the new/edited provider (unless the user explicitly skipped verification)
 * and rolls back on failure.
 */
import { useCallback } from 'react';

import {
  flushCloudProviders,
  listProviderModels,
  setCloudProviderKey,
} from '../../../../services/api/aiSettingsApi';
import { presentProviderSetupError } from '../ProviderSetupErrorNotice';
import {
  type AISettings,
  type CloudProvider,
  maskKeyLabel,
  ProviderProbeError,
} from './aiPanelTypes';

export function useCloudProviderEditorSubmit({
  editing,
  draft,
  saved,
  persist,
  t,
  onDone,
}: {
  editing: CloudProvider | 'new' | null;
  draft: AISettings;
  saved: AISettings;
  persist: (next: AISettings) => Promise<void>;
  t: (key: string, fallback?: string) => string;
  onDone: () => void;
}) {
  return useCallback(
    async (next: CloudProvider, apiKey: string, opts?: { skipProbe?: boolean }) => {
      const id =
        editing === 'new' || !editing?.id
          ? `p_${next.slug}_${Math.random().toString(36).slice(2, 7)}`
          : editing.id;
      const upserted: CloudProvider = {
        ...next,
        id,
        maskedKey: maskKeyLabel(apiKey ? true : next.maskedKey.startsWith('••••')),
      };

      // Snapshot the prior persisted cloud_providers list so we can restore
      // it if the live probe fails.
      const priorWireProviders = saved.cloudProviders.map(p => ({
        id: p.id,
        slug: p.slug,
        label: p.label,
        endpoint: p.endpoint,
        image_endpoint: p.imageEndpoint ?? '',
        model_image_endpoints: p.modelImageEndpoints ?? {},
        model_transports: p.modelTransports ?? {},
        rotation_strategy: p.rotationStrategy ?? 'round_robin',
        tool_call_codec: p.toolCallCodec ?? 'native',
        ca_cert_pem: p.caCertPem ?? '',
        auth_style: p.authStyle,
      }));

      // The provider is provisionally flushed only so the native probe can
      // resolve its CA certificate, endpoint and auth style. The current API
      // key remains in encrypted storage until the candidate has passed.
      const list =
        editing === 'new' || !editing
          ? [...draft.cloudProviders, upserted]
          : draft.cloudProviders.map(p => (p.id === editing.id ? upserted : p));
      if (upserted.slug === 'openhuman') {
        await persist({ ...draft, cloudProviders: list });
        onDone();
        return;
      }

      const nextWireProviders = list
        .filter(p => !['', 'cloud', 'openhuman', 'pid'].includes(p.slug))
        .map(p => ({
          id: p.id,
          slug: p.slug,
          label: p.label,
          endpoint: p.endpoint,
          image_endpoint: p.imageEndpoint ?? '',
          model_image_endpoints: p.modelImageEndpoints ?? {},
          model_transports: p.modelTransports ?? {},
          rotation_strategy: p.rotationStrategy ?? 'round_robin',
          tool_call_codec: p.toolCallCodec ?? 'native',
          ca_cert_pem: p.caCertPem ?? '',
          auth_style: p.authStyle,
        }));

      let settingsPersisted = false;
      try {
        await flushCloudProviders(nextWireProviders);
        // A candidate credential is sent for this one-shot native read-only
        // probe but is neither persisted nor mixed with the saved key fleet.
        // Failed probes therefore leave the previous encrypted key untouched.
        if (!opts?.skipProbe) {
          try {
            await listProviderModels(upserted.slug, apiKey || undefined);
          } catch (probeErr) {
            const msg = probeErr instanceof Error ? probeErr.message : String(probeErr);
            console.warn('[ai-settings] provider /models probe failed', {
              slug: upserted.slug,
              summary: presentProviderSetupError(msg, t).summary,
            });
            throw new ProviderProbeError(`Could not reach ${upserted.label}: ${msg}.`);
          }
        }

        // Persist the provider and other settings first. A failed settings
        // write must not overwrite a working credential; the AuthService key
        // replacement is the last irreversible operation in this flow.
        await persist({ ...draft, cloudProviders: list });
        settingsPersisted = true;
        if (apiKey) {
          await setCloudProviderKey(upserted.slug, apiKey);
        }
      } catch (error) {
        await flushCloudProviders(priorWireProviders).catch(rollbackErr =>
          console.warn(
            `[ai-settings] rollback provider settings failed slug=${upserted.slug}`,
            rollbackErr
          )
        );
        if (settingsPersisted) {
          // Restore the editor's saved/draft snapshot after a failed key-store
          // write. This keeps the UI consistent with the reverted provider.
          await persist(saved).catch(rollbackErr =>
            console.warn(
              `[ai-settings] rollback provider editor state failed slug=${upserted.slug}`,
              rollbackErr
            )
          );
        }
        throw error;
      }
      onDone();
    },
    [editing, draft, saved, persist, t, onDone]
  );
}
