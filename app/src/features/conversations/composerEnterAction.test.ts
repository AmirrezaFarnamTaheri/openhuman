import { describe, expect, it } from 'vitest';

import {
  COMPOSER_ENTER_ACTIONS,
  type ComposerEnterKeyEvent,
  isComposerImeComposing,
  parseComposerEnterAction,
  resolveComposerEnterLane,
  shouldInsertNewline,
} from './composerEnterAction';

// `key` comes last so the helper's return type is exactly
// `ComposerEnterKeyEvent` — the spread of a `Partial` would otherwise make
// `key` possibly-undefined.
const enter = (overrides: Partial<ComposerEnterKeyEvent> = {}): ComposerEnterKeyEvent => ({
  ...overrides,
  key: 'Enter',
});

describe('parseComposerEnterAction', () => {
  it('accepts every known action', () => {
    for (const action of COMPOSER_ENTER_ACTIONS) {
      expect(parseComposerEnterAction(action)).toBe(action);
    }
  });

  it('falls back to send for anything unknown', () => {
    expect(parseComposerEnterAction('nonsense')).toBe('send');
    expect(parseComposerEnterAction(undefined)).toBe('send');
    expect(parseComposerEnterAction(null)).toBe('send');
    expect(parseComposerEnterAction(42)).toBe('send');
  });
});

describe('isComposerImeComposing', () => {
  it('detects a composition from any of the signals the browser gives', () => {
    expect(isComposerImeComposing({ key: 'Enter', isComposing: true })).toBe(true);
    expect(isComposerImeComposing({ key: 'Enter', keyCode: 229 })).toBe(true);
    expect(isComposerImeComposing({ key: 'Enter', nativeEvent: { isComposing: true } })).toBe(true);
    expect(isComposerImeComposing({ key: 'Enter', nativeEvent: { keyCode: 229 } })).toBe(true);
    expect(isComposerImeComposing({ key: 'Enter' }, true)).toBe(true);
  });

  it('is false for an ordinary Enter', () => {
    expect(isComposerImeComposing({ key: 'Enter' })).toBe(false);
    expect(isComposerImeComposing({ key: 'Enter' }, false)).toBe(false);
  });
});

describe('resolveComposerEnterLane', () => {
  it('sends on a plain Enter by default', () => {
    expect(resolveComposerEnterLane(enter(), { enterAction: 'send', isRunning: false })).toBe(
      'send'
    );
  });

  it('sends on a plain Enter while a turn is running (a follow-up)', () => {
    expect(resolveComposerEnterLane(enter(), { enterAction: 'send', isRunning: true })).toBe(
      'send'
    );
  });

  it('steers on Ctrl+Enter and on Cmd+Enter', () => {
    expect(
      resolveComposerEnterLane(enter({ ctrlKey: true }), { enterAction: 'send', isRunning: true })
    ).toBe('steer');
    expect(
      resolveComposerEnterLane(enter({ metaKey: true }), { enterAction: 'send', isRunning: true })
    ).toBe('steer');
  });

  it('queues on Alt+Enter', () => {
    expect(
      resolveComposerEnterLane(enter({ altKey: true }), { enterAction: 'send', isRunning: true })
    ).toBe('queue');
  });

  it('keeps the modifier lanes fixed even when plain Enter is configured', () => {
    for (const enterAction of COMPOSER_ENTER_ACTIONS) {
      expect(
        resolveComposerEnterLane(enter({ ctrlKey: true }), { enterAction, isRunning: true })
      ).toBe('steer');
      expect(
        resolveComposerEnterLane(enter({ altKey: true }), { enterAction, isRunning: true })
      ).toBe('queue');
    }
  });

  it('never sends on Shift+Enter', () => {
    for (const enterAction of COMPOSER_ENTER_ACTIONS) {
      expect(
        resolveComposerEnterLane(enter({ shiftKey: true }), { enterAction, isRunning: true })
      ).toBeNull();
    }
  });

  it('never sends while an IME composition is open', () => {
    expect(
      resolveComposerEnterLane(enter({ isComposing: true }), {
        enterAction: 'send',
        isRunning: false,
      })
    ).toBeNull();
    expect(
      resolveComposerEnterLane(enter({ nativeEvent: { keyCode: 229 } }), {
        enterAction: 'send',
        isRunning: false,
      })
    ).toBeNull();
    expect(
      resolveComposerEnterLane(enter(), {
        enterAction: 'send',
        isRunning: false,
        compositionActive: true,
      })
    ).toBeNull();
  });

  it('queues on plain Enter when the action is queue', () => {
    expect(resolveComposerEnterLane(enter(), { enterAction: 'queue', isRunning: false })).toBe(
      'queue'
    );
    expect(resolveComposerEnterLane(enter(), { enterAction: 'queue', isRunning: true })).toBe(
      'queue'
    );
  });

  it('steers on plain Enter while running, and degrades to a send when idle', () => {
    expect(resolveComposerEnterLane(enter(), { enterAction: 'steer', isRunning: true })).toBe(
      'steer'
    );
    expect(resolveComposerEnterLane(enter(), { enterAction: 'steer', isRunning: false })).toBe(
      'send'
    );
  });

  it('never sends when plain Enter is set to newline', () => {
    expect(
      resolveComposerEnterLane(enter(), { enterAction: 'newline', isRunning: false })
    ).toBeNull();
    expect(
      resolveComposerEnterLane(enter(), { enterAction: 'newline', isRunning: true })
    ).toBeNull();
  });

  it('ignores keys that are not Enter', () => {
    expect(
      resolveComposerEnterLane({ key: 'a' }, { enterAction: 'send', isRunning: false })
    ).toBeNull();
  });
});

describe('shouldInsertNewline', () => {
  it('inserts a newline only for the newline action', () => {
    expect(shouldInsertNewline(enter(), { enterAction: 'newline' })).toBe(true);
    expect(shouldInsertNewline(enter(), { enterAction: 'send' })).toBe(false);
    expect(shouldInsertNewline(enter(), { enterAction: 'queue' })).toBe(false);
    expect(shouldInsertNewline(enter(), { enterAction: 'steer' })).toBe(false);
  });

  it('leaves Shift+Enter to the editor', () => {
    expect(shouldInsertNewline(enter({ shiftKey: true }), { enterAction: 'newline' })).toBe(false);
  });

  it('does not insert a newline for the modifier lanes', () => {
    expect(shouldInsertNewline(enter({ ctrlKey: true }), { enterAction: 'newline' })).toBe(false);
    expect(shouldInsertNewline(enter({ altKey: true }), { enterAction: 'newline' })).toBe(false);
  });

  it('does not insert a newline mid-composition', () => {
    expect(shouldInsertNewline(enter({ isComposing: true }), { enterAction: 'newline' })).toBe(
      false
    );
  });
});
