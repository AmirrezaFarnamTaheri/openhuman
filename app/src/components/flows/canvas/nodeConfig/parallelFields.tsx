/**
 * Native TinyFlows scatter/gather controls. Scatter executes a downstream
 * pipeline once per lane, up to the engine's MAX_LANES (256). Gather collects
 * lane outputs in original index order using the engine's release policy.
 *
 * Config mirrors the pinned tinyflows node executors; do not invent a second
 * map/reduce scheduler or store. Other uncommon fields remain editable via
 * the node drawer's existing raw JSON escape hatch.
 */
import { useT } from '../../../../lib/i18n/I18nContext';
import {
  configNumber, configString, NumberField, SelectField, TextAreaField, TextField,
} from './nodeConfigFields';

interface ParallelFormProps {
  config: Record<string, unknown>;
  onChange: (patch: Record<string, unknown>) => void;
}

export function ScatterForm({ config, onChange }: ParallelFormProps) {
  const { t } = useT();
  return (
    <div className="space-y-3">
      <TextField
        label={t('flows.nodeConfig.scatter.pathLabel')}
        hint={t('flows.nodeConfig.scatter.pathHint')}
        value={configString(config, 'path')}
        onChange={value => onChange({ path: value })}
        placeholder="rows"
        testId="node-config-scatter-path"
      />
      <NumberField
        label={t('flows.nodeConfig.scatter.lanesLabel')}
        hint={t('flows.nodeConfig.scatter.lanesHint')}
        value={configNumber(config, 'lanes')}
        onChange={value => onChange({ lanes: value })}
        min={1}
        max={256}
        step={1}
        testId="node-config-scatter-lanes"
      />
    </div>
  );
}

export function GatherForm({ config, onChange }: ParallelFormProps) {
  const { t } = useT();
  const release = configString(config, 'release') || 'all';
  const laneError = configString(config, 'on_lane_error') || 'collect';
  const from = Array.isArray(config.from)
    ? config.from.filter((id): id is string => typeof id === 'string').join('\n')
    : '';
  return (
    <div className="space-y-3">
      <TextAreaField
        label={t('flows.nodeConfig.gather.sourcesLabel')}
        hint={t('flows.nodeConfig.gather.sourcesHint')}
        value={from}
        rows={3}
        mono
        onChange={value => onChange({
          from: [...new Set(value.split(/[\n,]/).map(id => id.trim()).filter(Boolean))],
        })}
        testId="node-config-gather-sources"
      />
      <SelectField
        label={t('flows.nodeConfig.gather.releaseLabel')}
        value={release}
        onChange={value => onChange(
          value === 'first_n' || value === 'quorum'
            ? { release: value, n: configNumber(config, 'n') ?? 1 }
            : { release: value }
        )}
        testId="node-config-gather-release"
        options={[
          { value: 'all', label: t('flows.nodeConfig.gather.release_all') },
          { value: 'any', label: t('flows.nodeConfig.gather.release_any') },
          { value: 'first_n', label: t('flows.nodeConfig.gather.release_first_n') },
          { value: 'quorum', label: t('flows.nodeConfig.gather.release_quorum') },
          { value: 'timeout_partial', label: t('flows.nodeConfig.gather.release_timeout_partial') },
        ]}
      />
      {(release === 'first_n' || release === 'quorum') && (
        <NumberField
          label={t('flows.nodeConfig.gather.arrivalsLabel')}
          value={configNumber(config, 'n') ?? 1}
          onChange={value => onChange({ n: value })}
          min={1}
          step={1}
          testId="node-config-gather-required-arrivals"
        />
      )}
      <SelectField
        label={t('flows.nodeConfig.gather.laneErrorLabel')}
        value={laneError}
        onChange={value => onChange({ on_lane_error: value })}
        testId="node-config-gather-lane-error"
        options={[
          { value: 'collect', label: t('flows.nodeConfig.gather.error_collect') },
          { value: 'skip', label: t('flows.nodeConfig.gather.error_skip') },
          { value: 'fail_fast', label: t('flows.nodeConfig.gather.error_fail_fast') },
        ]}
      />
      <NumberField
        label={t('flows.nodeConfig.gather.maxPollsLabel')}
        value={configNumber(config, 'max_polls') ?? 500}
        onChange={value => onChange({ max_polls: value })}
        min={1}
        step={1}
        testId="node-config-gather-max-polls"
      />
    </div>
  );
}
