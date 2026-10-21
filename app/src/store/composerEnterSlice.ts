/**
 * What the Enter key does in the chat composer.
 *
 * A small, persisted user preference: it is a property of the person using
 * the app, not of a conversation, so it lives app-wide rather than per thread.
 * The decision itself (which lane a key press asks for) is a pure function in
 * `features/conversations/composerEnterAction.ts`; this slice only holds the
 * setting.
 */
import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

import {
  type ComposerEnterAction,
  parseComposerEnterAction,
} from '../features/conversations/composerEnterAction';

export interface ComposerEnterState {
  /** What plain Enter does. Defaults to `'send'`. */
  enterAction: ComposerEnterAction;
}

const initialState: ComposerEnterState = { enterAction: 'send' };

const composerEnterSlice = createSlice({
  name: 'composerEnter',
  initialState,
  reducers: {
    setComposerEnterAction: (state, action: PayloadAction<ComposerEnterAction>) => {
      state.enterAction = parseComposerEnterAction(action.payload);
    },
  },
});

export const { setComposerEnterAction } = composerEnterSlice.actions;
export default composerEnterSlice.reducer;
