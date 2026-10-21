/**
 * What the Enter key does in the chat composer.
 *
 * The composer had one behaviour: Enter sends, Shift+Enter inserts a newline.
 * That is not enough once a conversation can have a turn in flight — the user
 * then needs three more affordances, and they conflict:
 *
 * - **send** — hand the message to the core as a fresh turn.
 * - **queue** — hold it behind the running turn (`queue_mode: 'followup'`).
 * - **steer** — inject it into the running turn at its next iteration
 *   boundary (`queue_mode: 'steer'`).
 *
 * The core already speaks all three (`chatSend`'s `queueMode`), and the
 * composer already routes a send while a turn streams to the queue lane. What
 * was missing was a way for the *user* to pick a lane with the keyboard, and a
 * way to change what plain Enter means.
 *
 * ## The contract
 *
 * Modifiers are fixed, so the user can rely on them without configuring
 * anything:
 *
 * | Keys            | Action                                   |
 * | --------------- | ---------------------------------------- |
 * | Enter           | whatever `enterAction` says (default send) |
 * | Ctrl/Cmd+Enter  | steer (interrupt the running turn's next step) |
 * | Alt+Enter       | queue (hold for after the running turn)  |
 * | Shift+Enter     | newline (never sends)                    |
 *
 * Ctrl/Cmd+Enter and Alt+Enter are deliberately *not* configurable: they are
 * the two lanes a running turn needs, and a setting that can disable them
 * leaves the user with no way to reach them.
 *
 * `enterAction` customises plain Enter only:
 *
 * - `send` — the default, and what an idle conversation wants.
 * - `queue` — plain Enter always queues, even when nothing is running. The
 *   queue is drained by the next turn, so a user who prefers never
 *   interrupting can set this once.
 * - `steer` — plain Enter steers a running turn; on an idle conversation there
 *   is nothing to steer, so it degrades to a send.
 * - `newline` — Enter inserts a newline and never sends. The Send button
 *   remains the only way to send, which is the right choice for people who
 *   write multi-paragraph prompts.
 *
 * ## Why this is a pure module
 *
 * The decision is a function of the event and the setting, with no I/O, so it
 * is testable without a DOM and reusable by every composer surface (the
 * assistant-ui chat, the legacy text composer, the workflow copilot). The
 * IME rules are the subtle part and belong next to the decision rather than
 * in a component: an Enter that is part of an IME composition is the IME's
 * business, never a send.
 */

/** What plain Enter does in the composer. */
export type ComposerEnterAction = 'send' | 'queue' | 'steer' | 'newline';

/** The lanes a message can travel in. `send` is not a lane — it is a turn. */
export type ComposerSendLane = 'send' | 'queue' | 'steer';

/** Every value `enterAction` may hold, in the order the settings UI lists them. */
export const COMPOSER_ENTER_ACTIONS: readonly ComposerEnterAction[] = [
  'send',
  'queue',
  'steer',
  'newline',
];

const ENTER_ACTIONS = new Set<string>(COMPOSER_ENTER_ACTIONS);

/** Narrows an untrusted value (persisted state, an old blob) to a known action. */
export function parseComposerEnterAction(value: unknown): ComposerEnterAction {
  return typeof value === 'string' && ENTER_ACTIONS.has(value)
    ? (value as ComposerEnterAction)
    : 'send';
}

export interface ComposerEnterKeyEvent {
  key: string;
  shiftKey?: boolean;
  altKey?: boolean;
  ctrlKey?: boolean;
  metaKey?: boolean;
  /** True while an IME candidate window is open. */
  isComposing?: boolean;
  keyCode?: number;
  nativeEvent?: { isComposing?: boolean; keyCode?: number };
}

/**
 * True when the Enter key belongs to an IME composition.
 *
 * A composition's Enter confirms the candidate word. Treating it as a send
 * both sends a half-finished word and cancels the composition, which is the
 * bug the composer's own guard already worked around for the legacy
 * textarea; the same rule has to hold here.
 */
export function isComposerImeComposing(
  event: ComposerEnterKeyEvent,
  compositionActive = false
): boolean {
  return (
    compositionActive ||
    event.isComposing === true ||
    event.keyCode === 229 ||
    event.nativeEvent?.isComposing === true ||
    event.nativeEvent?.keyCode === 229
  );
}

/**
 * The lane an Enter press asks for, or `null` when the key is not a send
 * at all (a newline, an IME composition, or a key that is not Enter).
 *
 * `isRunning` is the thread's state, not the event's: it is what turns the
 * steer lane into a send when there is no turn to steer.
 */
export function resolveComposerEnterLane(
  event: ComposerEnterKeyEvent,
  options: {
    /** What plain Enter means. */
    enterAction: ComposerEnterAction;
    /** Whether a turn is currently in flight for this conversation. */
    isRunning: boolean;
    /** True while an IME composition is open. */
    compositionActive?: boolean;
  }
): ComposerSendLane | null {
  if (event.key !== 'Enter') return null;
  if (isComposerImeComposing(event, options.compositionActive)) return null;
  // Shift+Enter is a newline in every configuration. It is the one key that
  // never sends, because "type a paragraph, then send" is universal.
  if (event.shiftKey) return null;

  // Ctrl/Cmd+Enter steers, Alt+Enter queues. Both are fixed.
  if (event.ctrlKey || event.metaKey) return 'steer';
  if (event.altKey) return 'queue';

  switch (options.enterAction) {
    case 'newline':
      return null;
    case 'queue':
      return 'queue';
    case 'steer':
      // Nothing to steer on an idle conversation, so the lane degrades to a
      // send rather than dropping the message.
      return options.isRunning ? 'steer' : 'send';
    case 'send':
    default:
      // A plain send on a running turn is a follow-up, which is what the
      // composer has always done.
      return 'send';
  }
}

/**
 * Whether the composer should insert a newline itself for this key press.
 *
 * Only the `newline` action needs it: Shift+Enter is handled by the editor
 * itself, and every other action consumes the key as a send.
 */
export function shouldInsertNewline(
  event: ComposerEnterKeyEvent,
  options: { enterAction: ComposerEnterAction; compositionActive?: boolean }
): boolean {
  if (event.key !== 'Enter') return false;
  if (isComposerImeComposing(event, options.compositionActive)) return false;
  if (event.shiftKey) return false;
  if (event.ctrlKey || event.metaKey || event.altKey) return false;
  return options.enterAction === 'newline';
}
