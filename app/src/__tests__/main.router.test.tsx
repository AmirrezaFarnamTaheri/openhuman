import { beforeEach, describe, expect, it, vi } from 'vitest';

const boot = vi.hoisted(() => ({
  resolveUser: null as ((id: string | null) => void) | null,
  resolveActiveUserBootstrap: vi.fn(),
  primeActiveUserId: vi.fn(),
  createAppDataRouter: vi.fn(),
  render: vi.fn(),
  disposeRouter: vi.fn(),
  isTauri: vi.fn(() => false),
  getCurrentWindow: vi.fn(() => ({ label: 'main' })),
}));

vi.mock('react-dom/client', () => ({
  default: { createRoot: vi.fn(() => ({ render: boot.render })) },
}));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: boot.getCurrentWindow }));
vi.mock('../App', () => ({ default: () => null, createAppDataRouter: boot.createAppDataRouter }));
vi.mock('../mascot/MascotWindowApp', () => ({ default: () => null }));
vi.mock('../notch/NotchApp', () => ({ default: () => null }));
vi.mock('../overlay/OverlayApp', () => ({ default: () => null }));
vi.mock('../polyfills', () => ({}));
vi.mock('../lib/autoHideScrollbars', () => ({ installAutoHideScrollbars: vi.fn() }));
vi.mock('../lib/coreState/store', () => ({ getCoreStateSnapshot: vi.fn() }));
vi.mock('../services/analytics', () => ({
  initGA: vi.fn(),
  initSentry: vi.fn(),
  startUiInteractionTracking: vi.fn(),
  trackEvent: vi.fn(),
}));
vi.mock('../services/apiClient', () => ({ setStoreForApiClient: vi.fn() }));
vi.mock('../store/userScopedStorage', () => ({ primeActiveUserId: boot.primeActiveUserId }));
vi.mock('../utils/bootstrapActiveUser', () => ({
  resolveActiveUserBootstrap: boot.resolveActiveUserBootstrap,
}));
vi.mock('../utils/config', () => ({ APP_VERSION: 'test' }));
vi.mock('../utils/configPersistence', () => ({ getStoredCoreMode: () => 'local' }));
vi.mock('../utils/desktopDeepLinkListener', () => ({
  setupDesktopDeepLinkListener: vi.fn(async () => undefined),
}));
vi.mock('../utils/tauriCommands', () => ({ getActiveUserIdFromCore: vi.fn() }));
vi.mock('../utils/tauriCommands/common', () => ({ isTauri: boot.isTauri }));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  window.history.replaceState(null, '', '/#/chat');
  document.body.innerHTML = '<div id="root"></div>';
  boot.isTauri.mockReturnValue(false);
  boot.getCurrentWindow.mockReturnValue({ label: 'main' });
  boot.resolveActiveUserBootstrap.mockImplementation(
    () => new Promise<string | null>(resolve => (boot.resolveUser = resolve))
  );
  boot.createAppDataRouter.mockImplementation(() => ({ dispose: boot.disposeRouter }));
});

describe('main data-router boot lifecycle', () => {
  it('normalizes the hash and primes the user before creating one router and rendering', async () => {
    window.history.replaceState(null, '', '/#unrecognized');
    await import('../main');

    expect(window.location.hash).toBe('#/');
    expect(boot.createAppDataRouter).not.toHaveBeenCalled();
    expect(boot.render).not.toHaveBeenCalled();

    boot.resolveUser?.('user-a');
    await vi.waitFor(() => expect(boot.render).toHaveBeenCalledOnce());

    expect(boot.primeActiveUserId).toHaveBeenCalledWith('user-a');
    expect(boot.createAppDataRouter).toHaveBeenCalledOnce();
    expect(boot.primeActiveUserId.mock.invocationCallOrder[0]).toBeLessThan(
      boot.createAppDataRouter.mock.invocationCallOrder[0]
    );
    expect(boot.createAppDataRouter.mock.invocationCallOrder[0]).toBeLessThan(
      boot.render.mock.invocationCallOrder[0]
    );
  });

  it('preserves a direct workflow entry while waiting for user bootstrap', async () => {
    window.history.replaceState(null, '', '/#/flows/draft?template=review');
    let initialHash: string | undefined;
    boot.createAppDataRouter.mockImplementation(() => {
      initialHash = window.location.hash;
      return { dispose: boot.disposeRouter };
    });
    await import('../main');
    boot.resolveUser?.('user-a');
    await vi.waitFor(() => expect(boot.render).toHaveBeenCalledOnce());

    expect(initialHash).toBe('#/flows/draft?template=review');
  });

  it.each(['mascot', 'notch', 'overlay'])(
    'does not create a main router for the %s window',
    async windowKind => {
      if (windowKind === 'overlay') {
        boot.isTauri.mockReturnValue(true);
        boot.getCurrentWindow.mockReturnValue({ label: 'overlay' });
      } else {
        window.history.replaceState(null, '', `/?window=${windowKind}`);
      }
      await import('../main');
      boot.resolveUser?.(null);
      await vi.waitFor(() => expect(boot.render).toHaveBeenCalledOnce());

      expect(boot.createAppDataRouter).not.toHaveBeenCalled();
    }
  );
});
