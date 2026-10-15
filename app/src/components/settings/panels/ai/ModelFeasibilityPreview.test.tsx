import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { ModelRegistryEntry } from '../../../../utils/tauriCommands/config';
import { ModelFeasibilityPreview } from './ModelFeasibilityPreview';

const cachedModel: ModelRegistryEntry = {
  provider: 'demo',
  id: 'cache-capable',
  context_window: 64_000,
  vision: false,
  cost_per_1m_input: 10,
  cost_per_1m_cached_input: 1,
  cost_per_1m_output: 20,
};

describe('ModelFeasibilityPreview cached input controls', () => {
  it('accounts for cached tokens as a subset of estimated input, without modifying routing', () => {
    render(<ModelFeasibilityPreview models={[cachedModel]} />);
    fireEvent.change(screen.getByLabelText('Estimated input tokens'), {
      target: { value: '1000' },
    });
    fireEvent.change(screen.getByLabelText('Cached input tokens (within input)'), {
      target: { value: '800' },
    });
    fireEvent.change(screen.getByLabelText('Estimated output tokens'), {
      target: { value: '100' },
    });
    expect(screen.getByText('Est. $0.0048')).toBeTruthy();
    expect(screen.getByText(/Matches saved metadata; availability unverified/)).toBeTruthy();
  });

  it('requires cached input to be no greater than total input tokens', () => {
    render(<ModelFeasibilityPreview models={[cachedModel]} />);
    fireEvent.change(screen.getByLabelText('Estimated input tokens'), {
      target: { value: '1000' },
    });
    fireEvent.change(screen.getByLabelText('Cached input tokens (within input)'), {
      target: { value: '1001' },
    });
    expect(screen.getByRole('alert').textContent).toContain(
      'Cached input must be within total input tokens'
    );
  });
});
