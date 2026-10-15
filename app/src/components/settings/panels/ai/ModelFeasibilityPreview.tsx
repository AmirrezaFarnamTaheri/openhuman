/** Read-only cost/capability estimator. Never changes model routing or secrets. */
import { useMemo, useState } from 'react';
import type { ModelRegistryEntry } from '../../../../utils/tauriCommands/config';
import { assessModelFeasibility, formatEstimatedModelCost, type ModelFitReason } from './modelFeasibility';

const reasonText: Record<ModelFitReason, string> = {
  context_too_small: 'Context smaller than input + output tokens',
  vision_unavailable: 'Vision unsupported in saved metadata',
  estimated_cost_exceeds_budget: 'Estimated cost exceeds budget',
  unknown_context: 'Context size unknown',
  unknown_vision: 'Vision capability unknown',
  unknown_price: 'Price missing, zero or invalid',
};

interface Props {
  models: readonly ModelRegistryEntry[];
}

export function ModelFeasibilityPreview({ models }: Props) {
  const [inputText, setInputText] = useState('32000');
  const [outputText, setOutputText] = useState('2000');
  const [budgetText, setBudgetText] = useState('');
  const [requiresVision, setRequiresVision] = useState(false);

  const report = useMemo(() => {
    if (!inputText.trim() || !outputText.trim()) return null;
    try {
      return assessModelFeasibility(models, {
        inputTokens: Number(inputText),
        outputTokens: Number(outputText),
        requiresVision,
        maxEstimatedCostUSD: budgetText.trim() === '' ? null : Number(budgetText),
      });
    } catch {
      return null;
    }
  }, [models, inputText, outputText, budgetText, requiresVision]);

  const displayed = report?.rows.slice(0, 8) ?? [];
  return (
    <section aria-label="Model suitability preview"
      className="space-y-3 rounded-lg border border-line-subtle p-3">
      <div>
        <h4 className="text-sm font-medium text-content">Model suitability and cost preview</h4>
        <p className="text-xs text-content-muted">
          Based on saved metadata, not verified live prices, quotas, keys or availability.
          This is advisory only and does not change any workload route.
        </p>
      </div>
      <div className="grid grid-cols-2 gap-2 md:grid-cols-4">
        <label className="space-y-1 text-xs text-content-secondary">
          <span>Estimated input tokens</span>
          <input type="number" min="0" step="1" value={inputText}
            aria-label="Estimated input tokens" onChange={e => setInputText(e.target.value)}
            className="w-full rounded border border-line-subtle bg-transparent px-2 py-1 text-content" />
        </label>
        <label className="space-y-1 text-xs text-content-secondary">
          <span>Estimated output tokens</span>
          <input type="number" min="0" step="1" value={outputText}
            aria-label="Estimated output tokens" onChange={e => setOutputText(e.target.value)}
            className="w-full rounded border border-line-subtle bg-transparent px-2 py-1 text-content" />
        </label>
        <label className="space-y-1 text-xs text-content-secondary">
          <span>Cost limit (USD, optional)</span>
          <input type="number" min="0" step="0.01" value={budgetText}
            aria-label="Estimated cost limit in USD" placeholder="No limit"
            onChange={e => setBudgetText(e.target.value)}
            className="w-full rounded border border-line-subtle bg-transparent px-2 py-1 text-content" />
        </label>
        <label className="flex items-center gap-2 text-xs text-content-secondary">
          <input type="checkbox" checked={requiresVision}
            onChange={e => setRequiresVision(e.target.checked)} />
          <span>Require vision</span>
        </label>
      </div>
      {!report ? (
        <p role="alert" className="text-xs text-content-secondary">
          Enter non-negative whole token counts and an optional non-negative cost limit.
        </p>
      ) : (
        <>
          <p role="status" className="text-xs text-content-secondary">
            Saved-metadata fit: {report.counts.metadata_fit}; unknown: {report.counts.unknown};
            excluded by supplied constraints: {report.counts.excluded}.
          </p>
          <div className="max-h-60 space-y-1 overflow-y-auto">
            {displayed.map(({ model, estimatedCostUSD, reasons, status }) => (
              <div key={model.provider + '\0' + model.id}
                className="flex flex-wrap items-start justify-between gap-2 border-b border-line-subtle py-1.5 text-xs">
                <div className="min-w-0 flex-1">
                  <div className="break-all font-medium text-content">{model.provider}/{model.id}</div>
                  <div className="text-content-muted">
                    {status === 'metadata_fit'
                      ? 'Matches saved metadata; availability unverified'
                      : status === 'unknown' ? 'Unknown eligibility' : 'Excluded by constraints'}
                    {reasons.length > 0 && ': ' + reasons.map(reason => reasonText[reason]).join('; ')}
                  </div>
                </div>
                <div className="shrink-0 text-content-secondary">
                  {formatEstimatedModelCost(estimatedCostUSD)}
                </div>
              </div>
            ))}
            {displayed.length === 0 && (
              <p className="text-xs text-content-muted">No saved models to evaluate.</p>
            )}
          </div>
          {report.rows.length > displayed.length && (
            <p className="text-xs text-content-muted">
              Showing {displayed.length} of {report.rows.length} saved model entries.
            </p>
          )}
        </>
      )}
    </section>
  );
}
