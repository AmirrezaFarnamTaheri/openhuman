import { Navigate, Route, Routes } from 'react-router-dom';

import OnboardingLayout from './OnboardingLayout';
import CustomInferencePage from './pages/CustomInferencePage';
import CustomSearchPage from './pages/CustomSearchPage';
import VaultSetupStep from './pages/VaultSetupStep';
import WelcomePage from './pages/WelcomePage';

/**
 * Routed onboarding flow.
 *
 *   welcome ─┬─ TinyHumans session → completeAndExit → /chat
 *            └─ local session      → /custom/inference → search → vault → /chat
 *
 * `runtime-choice` is gone: the identity question is asked once on the
 * Welcome screen (`pages/Welcome.tsx`) as two cards, so asking it again three
 * screens later was the same question twice. Its page and step files stay on
 * disk, unrouted.
 *
 * Local steps expose configuration directly, without a repeated
 * Default-versus-Configure choice. Optional search can be skipped.
 *
 * Voice, OAuth and embeddings are retired: six steps before a first message
 * was a forced tour of Settings, and all three stay reachable from Settings.
 * Their page files are kept unrouted so reviving a step is a route entry
 * rather than a rewrite. (An older note here claimed the same for the
 * Gmail/Composio and context steps, but those files are gone — the claim was
 * stale, so treat keeping these as a choice to confirm, not a convention.)
 */
const Onboarding = () => {
  return (
    <Routes>
      <Route element={<OnboardingLayout />}>
        <Route index element={<Navigate to="welcome" replace />} />
        <Route path="welcome" element={<WelcomePage />} />
        <Route path="custom/inference" element={<CustomInferencePage />} />
        <Route path="custom/search" element={<CustomSearchPage />} />
        <Route path="custom/vault" element={<VaultSetupStep />} />
        <Route path="*" element={<Navigate to="welcome" replace />} />
      </Route>
    </Routes>
  );
};

export default Onboarding;
