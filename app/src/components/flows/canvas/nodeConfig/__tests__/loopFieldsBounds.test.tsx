import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { LoopForm } from '../loopFields';

describe('LoopForm bounded controller', () => {
  it('reflects the graph validator hard iteration cap in the numeric control', () => {
    const onChange = vi.fn();
    render(<LoopForm config={{ max_iterations: 25 }} onChange={onChange} />);
    const input = screen.getByTestId('node-config-loop-max-iterations');
    expect(input).toHaveAttribute('min', '1');
    expect(input).toHaveAttribute('max', '100');
    fireEvent.change(input, { target: { value: '100' } });
    expect(onChange).toHaveBeenCalledWith({ max_iterations: 100 });
  });

  it('shows the actual engine default instead of an empty iteration limit', () => {
    render(<LoopForm config={{}} onChange={vi.fn()} />);
    expect(screen.getByTestId('node-config-loop-max-iterations')).toHaveValue(25);
  });
});
