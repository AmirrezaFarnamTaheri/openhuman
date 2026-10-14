import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { GatherForm, ScatterForm } from '../parallelFields';

describe('native scatter/gather configuration', () => {
  it('limits parallel work to the native 256-lane ceiling', () => {
    const onChange = vi.fn();
    render(<ScatterForm config={{ lanes: 8 }} onChange={onChange} />);
    expect(screen.getByTestId('node-config-scatter-lanes')).toHaveAttribute('max', '256');
    fireEvent.change(screen.getByTestId('node-config-scatter-lanes'), {
      target: { value: '12' },
    });
    expect(onChange).toHaveBeenCalledWith({ lanes: 12 });
    fireEvent.change(screen.getByTestId('node-config-scatter-path'), {
      target: { value: 'records' },
    });
    expect(onChange).toHaveBeenCalledWith({ path: 'records' });
  });

  it('persists the exact upstream lane-terminal node IDs rather than port labels', () => {
    const onChange = vi.fn();
    render(<GatherForm config={{ from: ['a'] }} onChange={onChange} />);
    fireEvent.change(screen.getByTestId('node-config-gather-sources'), {
      target: { value: 'a\nb, a\nc' },
    });
    expect(onChange).toHaveBeenCalledWith({ from: ['a', 'b', 'c'] });
  });

  it('adds an executable minimum release count when selecting quorum', () => {
    const onChange = vi.fn();
    const { rerender } = render(<GatherForm config={{}} onChange={onChange} />);
    fireEvent.change(screen.getByTestId('node-config-gather-release'), {
      target: { value: 'quorum' },
    });
    expect(onChange).toHaveBeenCalledWith({ release: 'quorum', n: 1 });
    rerender(<GatherForm config={{ release: 'quorum', n: 2 }} onChange={onChange} />);
    expect(screen.getByTestId('node-config-gather-required-arrivals')).toHaveValue(2);
    fireEvent.change(screen.getByTestId('node-config-gather-required-arrivals'), {
      target: { value: '3' },
    });
    expect(onChange).toHaveBeenCalledWith({ n: 3 });
  });

  it('exposes the native fail-fast and bounded polling controls', () => {
    const onChange = vi.fn();
    render(<GatherForm config={{}} onChange={onChange} />);
    expect(screen.getByTestId('node-config-gather-max-polls')).toHaveValue(500);
    fireEvent.change(screen.getByTestId('node-config-gather-lane-error'), {
      target: { value: 'fail_fast' },
    });
    expect(onChange).toHaveBeenCalledWith({ on_lane_error: 'fail_fast' });
  });
});
