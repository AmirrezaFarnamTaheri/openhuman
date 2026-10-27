import type { ReactNode } from 'react';
import { createHashRouter } from 'react-router-dom';

/** Keep hash deep links while enabling the editor's route-blocking API. */
export function createAppDataRouter(element: ReactNode) {
  return createHashRouter([{ path: '*', element }]);
}
