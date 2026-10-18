import { configureStore } from '@reduxjs/toolkit';
import { act, renderHook, waitFor } from '@testing-library/react';
import { type PropsWithChildren, StrictMode } from 'react';
import { Provider } from 'react-redux';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { Announcement } from '../../../services/announcementService';
import announcementReducer from '../../../store/announcementSlice';
import { useAnnouncementNotice } from '../useAnnouncementNotice';

const h = vi.hoisted(() => ({
  auth: { isAuthenticated: true, userId: 'u1' as string | null },
  fetch: vi.fn(),
  openUrl: vi.fn(),
}));
vi.mock('../../../providers/CoreStateProvider', () => ({
  useCoreState: () => ({ snapshot: { auth: h.auth } }),
}));
vi.mock('../../../services/announcementService', () => ({
  fetchLatestAnnouncement: () => h.fetch(),
}));
vi.mock('../../../utils/openUrl', () => ({ openUrl: (...args: unknown[]) => h.openUrl(...args) }));

const sample: Announcement = {
  id: 'a1',
  title: 'Hello',
  body: 'World',
  severity: 'INFO',
  cta: null,
  startsAt: null,
  expiresAt: null,
  createdAt: null,
};

function renderNotice(shownIds: string[] = []) {
  const store = configureStore({
    reducer: { announcement: announcementReducer },
    preloadedState: { announcement: { shownIds } },
  });
  function Wrapper({ children }: PropsWithChildren) {
    return (
      <StrictMode>
        <Provider store={store}>{children}</Provider>
      </StrictMode>
    );
  }
  return { store, ...renderHook(useAnnouncementNotice, { wrapper: Wrapper }) };
}

describe('useAnnouncementNotice', () => {
  beforeEach(() => {
    h.auth.isAuthenticated = true;
    h.auth.userId = 'u1';
    h.fetch.mockReset().mockResolvedValue(sample);
    h.openUrl.mockReset().mockResolvedValue(undefined);
  });

  it.each([
    ['INFO', 'info'],
    ['WARNING', 'warning'],
    ['CRITICAL', 'error'],
  ] as const)(
    'keeps %s severity and backend copy during effect replay',
    async (severity, expected) => {
      h.fetch.mockResolvedValue({ ...sample, severity });
      const { result } = renderNotice();
      await waitFor(() =>
        expect(result.current).toMatchObject({
          id: 'announcement:a1',
          title: 'Hello',
          body: 'World',
          severity: expected,
        })
      );
    }
  );

  it('does not fetch or show an announcement while signed out', () => {
    h.auth.isAuthenticated = false;
    const { result } = renderNotice();
    expect(result.current).toBeNull();
    expect(h.fetch).not.toHaveBeenCalled();
  });

  it('hides a persisted dismissal', async () => {
    const { result } = renderNotice(['a1']);
    await act(async () => {});
    expect(result.current).toBeNull();
  });

  it('persists dismissal without requiring navigation', async () => {
    const { result, store } = renderNotice();
    await waitFor(() => expect(result.current).not.toBeNull());
    act(() => result.current?.onDismiss?.());
    expect(store.getState().announcement.shownIds).toEqual(['a1']);
    expect(result.current).toBeNull();
  });

  it('does not expose the previous account announcement during an account switch', async () => {
    const { result, rerender } = renderNotice();
    await waitFor(() => expect(result.current).not.toBeNull());
    let finish!: (value: Announcement | null) => void;
    h.fetch.mockReturnValue(
      new Promise(resolve => {
        finish = resolve;
      })
    );
    h.auth.userId = 'u2';
    rerender();
    expect(result.current).toBeNull();
    await act(async () => finish(null));
    expect(result.current).toBeNull();
  });

  it('ignores a delayed response after sign-out', async () => {
    let finish!: (value: Announcement) => void;
    h.fetch.mockReturnValue(
      new Promise(resolve => {
        finish = resolve;
      })
    );
    const { result, rerender } = renderNotice();
    h.auth.isAuthenticated = false;
    rerender();
    await act(async () => finish(sample));
    expect(result.current).toBeNull();
  });

  it('treats a fetch failure as optional information', async () => {
    h.fetch.mockRejectedValue(new Error('offline'));
    const { result } = renderNotice();
    await act(async () => {});
    expect(result.current).toBeNull();
  });

  it('does not record a delayed CTA completion against a different account', async () => {
    h.fetch.mockResolvedValue({
      ...sample,
      cta: { label: 'Read more', url: 'https://example.com/news' },
    });
    let opened!: () => void;
    h.openUrl.mockReturnValue(
      new Promise<void>(resolve => {
        opened = resolve;
      })
    );
    const { result, store, rerender } = renderNotice();
    await waitFor(() => expect(result.current).not.toBeNull());
    act(() => result.current?.onAction?.());
    h.auth.userId = 'u2';
    h.fetch.mockResolvedValue(null);
    rerender();
    await act(async () => opened());
    expect(store.getState().announcement.shownIds).toEqual([]);
  });

  it('opens an external CTA and retains the notice for retry when opening fails', async () => {
    h.fetch.mockResolvedValue({
      ...sample,
      cta: { label: 'Read more', url: 'https://example.com/news' },
    });
    h.openUrl.mockRejectedValueOnce(new Error('OS handler refused'));
    const { result, store } = renderNotice();
    await waitFor(() => expect(result.current?.actionLabel).toBe('Read more'));
    await act(async () => result.current?.onAction?.());
    expect(h.openUrl).toHaveBeenCalledWith('https://example.com/news');
    expect(store.getState().announcement.shownIds).toEqual([]);
    expect(result.current).not.toBeNull();

    await act(async () => result.current?.onAction?.());
    expect(store.getState().announcement.shownIds).toEqual(['a1']);
    expect(result.current).toBeNull();
  });
});
