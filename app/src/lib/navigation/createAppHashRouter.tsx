import type { ReactNode } from 'react';
import { createHashRouter } from 'react-router-dom';

/**
 * The desktop shell needs a data-router-aware blocker while retaining hash
 * URLs for installed desktop/mobile applications. All routes are still owned
 * by the existing AppRoutes component rendered within this root shell.
 */
export function createAppHashRouter(shell: ReactNode) {
  return createHashRouter([{ path: '*', element: shell }]);
}
