/*
 * Submit handler for the advanced `CloudProviderEditor` modal — live-verifies
 * the new/edited provider (unless the user explicitly skipped verification)
 * and rolls back on failure.
 */
import { useCallback, useRef } from 'react';

import {
  flushCloudProviders,
  loadAISettings,
  listProviderModels,
  setCloudProviderKey,
} from '../../../../services/api/aiSettingsApi';
import { presentProviderSetupError } from '../ProviderSetupErrorNotice';
import { validModelFleetImageEndpoint } from './modelFleetOps';
import {
  type AISettings,
  type CloudProvider,
  maskKeyLabel,
  ProviderProbeError,
} from './aiPanelTypes';

/**
 * Compare only provider configuration, excluding encrypted-key presence and
 * unrelated routes/models. A compensating write is safe only when the live
 * provider list still equals our attempted write. This is a best-effort UI
 * safeguard, not a cross-process compare-and-swap or crash-atomic transaction.
 */
/** Never surface a candidate API key if a gateway echoes it in an error. */
function redactCandidateSecrets(message: string, rawKeys: string): string {
  const keys = [...new Set(rawKeys.split(/[,;\r\n]/).map(key => key.trim()).filter(Boolean))]
    .sort((a, b) => b.length - a.length);
  return keys.reduce((safe, secret) => safe.replaceAll(secret, '[redacted]'), message);
}

function providerListFingerprint(
  providers: ReadonlyArray<{
    id: string; slug: string; label: string; endpoint: string;
    auth_style: string; image_endpoint?: string | null;
    model_image_endpoints?: Record<string, string>;
    model_transports?: Record<string, string>;
    rotation_strategy?: string | null; tool_call_codec?: string; ca_cert_pem?: string | null;
  }>
): string {
  return JSON.stringify(providers
    .filter(p => !['', 'cloud', 'openhuman', 'pid'].includes(p.slug))
    .map(p => ({
      id: p.id, slug: p.slug, label: p.label, endpoint: p.endpoint,
      auth_style: p.auth_style,
      image_endpoint: p.image_endpoint ?? '',
      model_image_endpoints: p.model_image_endpoints ?? {},
      model_transports: p.model_transports ?? {},
      rotation_strategy: p.rotation_strategy ?? 'round_robin',
      tool_call_codec: p.tool_call_codec ?? 'native',
      ca_cert_pem: p.ca_cert_pem ?? '',
    })).sort((a, b) => a.slug.localeCompare(b.slug) || a.id.localeCompare(b.id)));
}

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
  // A ref guards the synchronous pre-render window as well as awaited RPCs.
  // It prevents overlapping saves from racing the compensation/key-store path.
  const saveInFlight = useRef(false);
  return useCallback(
    async (next: CloudProvider, apiKey: string, opts?: { skipProbe?: boolean }) => {
      if (saveInFlight.current) {
        throw new Error('A provider save is already in progress.');
      }
      saveInFlight.current = true;
      try {
      const id =
        editing === 'new' || !editing?.id
          ? `p_${next.slug}_${Math.random().toString(36).slice(2, 7)}`
          : editing.id;
      const upserted: CloudProvider = {
        ...next,
        id,
        maskedKey: maskKeyLabel(apiKey ? true : next.maskedKey.startsWith('••••')),
      };
      const previous = editing && editing !== 'new' ? editing : null;
      const savedKeyExists = previous?.maskedKey.startsWith('••••') ?? false;
      // Do not rely on the modal's form checks: the submit hook can be
      // invoked by other UI paths, retries or future automation callers.
      // Never send an existing or candidate key to off-loopback plaintext.
      // Preserve unauthenticated LAN-model endpoints for local deployments.
      if (upserted.slug !== 'openhuman') {
        const hasCredential = Boolean(apiKey.trim())
          || savedKeyExists
          || upserted.maskedKey.startsWith('••••');
        if (hasCredential && (
          !upserted.endpoint.trim() ||
          !validModelFleetImageEndpoint(upserted.endpoint)
        )) {
          throw new Error('API keys require HTTPS or loopback HTTP.');
        }
        const imageRoutes = [
          upserted.imageEndpoint ?? '',
          ...Object.values(upserted.modelImageEndpoints ?? {}),
        ];
        if (imageRoutes.some(url =>
          typeof url !== 'string' || !validModelFleetImageEndpoint(url)
        )) {
          throw new Error('Image endpoints require HTTPS or loopback HTTP.');
        }
      }
      const changedDestination = previous !== null && (
        previous.slug !== upserted.slug ||
        previous.endpoint !== upserted.endpoint ||
        previous.authStyle !== upserted.authStyle ||
        (previous.caCertPem ?? '') !== (upserted.caCertPem ?? '')
      );
      if (savedKeyExists && changedDestination && !apiKey) {
        // Even an explicit skipProbe must not route an existing saved key to a
        // newly configured endpoint or trusted CA without replacement.
        throw new Error(
          'Changing the provider endpoint or certificate requires a new API key. ' +
          'The previous saved credential cannot be forwarded to a new destination.'
        );
      }

      // Snapshot the prior persisted cloud_providers list so we can restore
      // it if the live probe fails.
      const priorWireProviders = saved.cloudProviders
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

      // Build a read-only preview; never re-point live traffic before the
      // proposed endpoint and candidate key have been verified.
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
      let settingsWriteAttempted = false;
      let replacementKeyWriteAttempted = false;
      // Detach any active old-key route before replacing a key for a new
      // destination. While detached, both an old key/new endpoint and a new
      // key/old endpoint combination are impossible through live config.
      const stagedRekey = savedKeyExists && changedDestination && Boolean(apiKey);
      const detachedProviders = stagedRekey
        ? list.filter(p => p.id !== previous?.id)
        : list;
      const detachedWireProviders = nextWireProviders.filter(
        p => p.id !== previous?.id
      );
      const previewProvider = nextWireProviders.find(p => p.slug === upserted.slug);
      if (!previewProvider) throw new Error('Provider preview entry not found');
      try {
        // A candidate credential is sent for this one-shot native read-only
        // probe but is neither persisted nor mixed with the saved key fleet.
        // Failed probes therefore leave the previous encrypted key untouched.
        if (!opts?.skipProbe && !(savedKeyExists && !changedDestination && !apiKey)) {
          try {
            await listProviderModels(upserted.slug, apiKey, previewProvider);
          } catch (probeErr) {
            const rawMessage = probeErr instanceof Error ? probeErr.message : String(probeErr);
            const msg = redactCandidateSecrets(rawMessage, apiKey);
            console.warn('[ai-settings] provider /models probe failed', {
              slug: upserted.slug,
              summary: presentProviderSetupError(msg, t).summary,
            });
            throw new ProviderProbeError(`Could not reach ${upserted.label}: ${msg}.`);
          }
        }

        if (stagedRekey) {
          // Safety first: remove the old provider from live config before
          // writing a key for a different destination. There is a brief,
          // explicit availability gap, but no cross-endpoint key exposure.
          settingsWriteAttempted = true;
          await persist({ ...draft, cloudProviders: detachedProviders });
          settingsPersisted = true;
          // A failed RPC does not prove the keyring write was rolled back.
          // From this point onward the old route must remain unavailable.
          replacementKeyWriteAttempted = true;
          await setCloudProviderKey(upserted.slug, apiKey);
          // Never restore the former endpoint after the new key has replaced
          // it. If final persistence fails, keep the provider detached.
          await persist({ ...draft, cloudProviders: list });
        } else {
          // New, non-retargeted providers keep the original config-first
          // order so an unsuccessful settings write preserves existing keys.
          settingsWriteAttempted = true;
          await persist({ ...draft, cloudProviders: list });
          settingsPersisted = true;
          if (apiKey) {
            await setCloudProviderKey(upserted.slug, apiKey);
          }
        }
      } catch (error) {
        // Do not replay a stale whole-list rollback over concurrent edits.
        // The provider snapshot is reloaded only after an attempted write.
        let safeToResetLocalState = false;
        let conflictOrUnknown = false;
        if (settingsWriteAttempted && !replacementKeyWriteAttempted) {
          try {
            const current = await loadAISettings();
            const observed = providerListFingerprint(current.cloudProviders);
            const prior = providerListFingerprint(priorWireProviders);
            const attempted = providerListFingerprint(
              stagedRekey ? detachedWireProviders : nextWireProviders
            );
            if (observed === attempted) {
              try {
                await flushCloudProviders(priorWireProviders);
                safeToResetLocalState = true;
              } catch {
                conflictOrUnknown = true;
                console.warn('[ai-settings] provider compensation failed; reload settings.');
              }
            } else if (observed === prior) {
              // A rejected write may have failed before reaching the core.
              safeToResetLocalState = true;
            } else {
              conflictOrUnknown = true;
              console.warn('[ai-settings] provider settings changed concurrently; rollback skipped.');
            }
          } catch {
            conflictOrUnknown = true;
            console.warn('[ai-settings] provider state could not be verified; rollback skipped.');
          }
        }
        if (replacementKeyWriteAttempted) {
          // A completed OR ambiguous key replacement cannot be undone without access to
          // the previous secret. Never restore the old provider destination:
          // doing so would forward the *new* key to the former endpoint.
          const detail = redactCandidateSecrets(
            error instanceof Error ? error.message : 'Provider save failed', apiKey
          );
          throw new Error(detail +
            '. The replacement-key write outcome is uncertain; the provider was detached ' +
            'to avoid cross-endpoint credential exposure. Reload settings and retry.');
        }
        if (settingsPersisted && safeToResetLocalState) {
          // Restore the UI snapshot only when core state is confirmed restored.
          await persist(saved).catch(() =>
            console.warn('[ai-settings] editor state reset failed; reload settings.')
          );
        }
        if (conflictOrUnknown) {
          const detail = redactCandidateSecrets(
            error instanceof Error ? error.message : 'Provider save failed', apiKey
          );
          throw new Error(detail + '. The provider state may have changed; reload settings.');
        }
        if (error instanceof Error && error.message !== redactCandidateSecrets(error.message, apiKey)) {
          throw new Error(redactCandidateSecrets(error.message, apiKey));
        }
        throw error;
      }
      onDone();
      } finally {
        saveInFlight.current = false;
      }
    },
    [editing, draft, saved, persist, t, onDone]
  );
}
