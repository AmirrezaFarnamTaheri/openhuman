import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { FlowRunItem } from '../../../lib/flows/runItems';
import { RunItemDataBrowser } from '../RunItemDataBrowser';

function manyItems(count: number): FlowRunItem[] {
  return Array.from({ length: count }, (_, index) => ({
    json: { position: index },
    binary: [],
    pairedIndex: null,
  }));
}

describe('RunItemDataBrowser large-result rendering', () => {
  it('renders the first 100 table rows and incrementally reveals more on demand', () => {
    const items = manyItems(251);
    render(<RunItemDataBrowser items={items} testIdPrefix="many" />);

    expect(screen.getByTestId('many-item-count')).toHaveTextContent('251');
    expect(screen.getAllByTestId(/^many-row-/)).toHaveLength(100);
    expect(screen.queryByTestId('many-row-100')).not.toBeInTheDocument();

    fireEvent.click(screen.getByTestId('many-load-more'));
    expect(screen.getAllByTestId(/^many-row-/)).toHaveLength(200);
    expect(screen.getByTestId('many-row-199')).toHaveTextContent('200');

    fireEvent.click(screen.getByTestId('many-load-more'));
    expect(screen.getAllByTestId(/^many-row-/)).toHaveLength(251);
    expect(screen.getByTestId('many-row-250')).toHaveTextContent('251');
    expect(screen.queryByTestId('many-load-more')).not.toBeInTheDocument();
  });

  it('keeps JSON view complete even before the table rows are expanded', () => {
    render(<RunItemDataBrowser items={manyItems(251)} testIdPrefix="json-large" />);
    expect(screen.queryByTestId('json-large-row-250')).not.toBeInTheDocument();
    fireEvent.click(screen.getByTestId('json-large-view-json'));
    expect(screen.getByTestId('json-large-json')).toHaveTextContent('"position": 250');
    expect(screen.queryByTestId('json-large-load-more')).not.toBeInTheDocument();
  });

  it('retains original source index after pagination', () => {
    const items = manyItems(140);
    items[122] = { json: { position: 122 }, binary: [], pairedIndex: 0 };
    render(
      <RunItemDataBrowser
        items={items}
        inputItems={[{ json: { source: 'original' }, binary: [], pairedIndex: null }]}
        testIdPrefix="paired"
      />
    );
    fireEvent.click(screen.getByTestId('paired-load-more'));
    fireEvent.click(screen.getByTestId('paired-source-toggle-122'));
    expect(screen.getByTestId('paired-source-122')).toHaveTextContent('original');
  });

  it('does not add a load-more control for small result sets', () => {
    render(<RunItemDataBrowser items={manyItems(2)} testIdPrefix="small" />);
    expect(screen.getAllByTestId(/^small-row-/)).toHaveLength(2);
    expect(screen.queryByTestId('small-load-more')).not.toBeInTheDocument();
  });
});
