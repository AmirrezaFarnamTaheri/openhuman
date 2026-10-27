import { describe, expect, it } from 'vitest';

import type { ComposerEnterAction } from '../features/conversations/composerEnterAction';
import reducer, { setComposerEnterAction } from './composerEnterSlice';

describe('composer Enter preference', () => {
  it.each<ComposerEnterAction>(['send', 'queue', 'steer', 'newline'])(
    'stores the supported %s preference',
    enterAction => {
      expect(reducer(undefined, setComposerEnterAction(enterAction)).enterAction).toBe(enterAction);
    }
  );

  it('defaults to send and retains the selected behavior across unrelated actions', () => {
    const initial = reducer(undefined, { type: '@@INIT' });
    expect(initial.enterAction).toBe('send');
    const newline = reducer(initial, setComposerEnterAction('newline'));
    expect(newline.enterAction).toBe('newline');
    expect(reducer(newline, { type: 'chat/otherAction' })).toBe(newline);
    expect(reducer(newline, setComposerEnterAction('send')).enterAction).toBe('send');
    expect(initial.enterAction).toBe('send');
  });

  it('normalizes an unrecognized persisted preference to send', () => {
    expect(
      reducer(undefined, setComposerEnterAction('unknown' as ComposerEnterAction)).enterAction
    ).toBe('send');
  });
});
