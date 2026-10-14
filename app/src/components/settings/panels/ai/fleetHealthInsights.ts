/** Privacy-safe readiness summary for the existing OpenHuman Rust model fleet.
 * Key health is a process-local observation, not provider quota or entitlement.
 * Inspired by Cockpit Tools' health dashboard and Antigravity Manager's quota UI.
 */
import type { ModelFleetStatus } from '../../../../services/api/aiSettingsApi';

export type FleetHealthSummary = {
  providerCount: number;
  modelCount: number;
  keyCount: number;
  healthy: number;
  cooling: number;
  evicted: number;
  activeLeases: number;
  nextRecoveryMs: number | null;
  status: 'unobserved' | 'ready' | 'degraded' | 'unavailable';
};

export function summarizeFleetHealth(snapshot: ModelFleetStatus | null): FleetHealthSummary {
  let providerCount = 0;
  let modelCount = 0;
  let keyCount = 0;
  let healthy = 0;
  let cooling = 0;
  let evicted = 0;
  let activeLeases = 0;
  let nextRecoveryMs: number | null = null;
  for (const provider of snapshot?.providers ?? []) {
    if (!provider || !Array.isArray(provider.models)) continue;
    providerCount++;
    for (const model of provider.models) {
      if (!model || !Array.isArray(model.keys)) continue;
      modelCount++;
      for (const key of model.keys) {
        if (!key || !['healthy', 'cooldown', 'evicted'].includes(key.status)) continue;
        keyCount++;
        if (key.status === 'healthy') healthy++;
        if (key.status === 'evicted') evicted++;
        if (key.status === 'cooldown') {
          cooling++;
          const retry = key.cooldown_remaining_ms;
          if (Number.isSafeInteger(retry) && retry > 0) {
            nextRecoveryMs = nextRecoveryMs === null ? retry : Math.min(nextRecoveryMs, retry);
          }
        }
        if (Number.isSafeInteger(key.active_leases) && key.active_leases >= 0) {
          activeLeases += key.active_leases;
        }
      }
    }
  }
  const status = keyCount === 0 ? 'unobserved'
    : healthy === 0 ? 'unavailable'
      : cooling + evicted > 0 ? 'degraded' : 'ready';
  return {providerCount, modelCount, keyCount, healthy, cooling, evicted,
    activeLeases, nextRecoveryMs, status};
}
