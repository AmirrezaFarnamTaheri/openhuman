import debugFactory from 'debug';
import { useEffect, useRef, useState } from 'react';

import { useCoreState } from '../../providers/CoreStateProvider';
import {
  type Announcement,
  type AnnouncementSeverity,
  fetchLatestAnnouncement,
} from '../../services/announcementService';
import { markAnnouncementShown } from '../../store/announcementSlice';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { openUrl } from '../../utils/openUrl';
import type { AppNotice, NoticeSeverity } from './useAppNotices';

const log = debugFactory('announcement');
const SEVERITY: Record<AnnouncementSeverity, NoticeSeverity> = {
  INFO: 'info',
  WARNING: 'warning',
  CRITICAL: 'error',
};

/**
 * Announcements join the notice center without interrupting the user's work.
 * Dismissal remains persisted per user. A response belongs to the identity
 * that requested it; changing accounts must never reveal the previous user's
 * announcement while the next request is pending or returns no announcement.
 */
export function useAnnouncementNotice(): AppNotice | null {
  const dispatch = useAppDispatch();
  const { snapshot } = useCoreState();
  const authKey = snapshot.auth.isAuthenticated ? (snapshot.auth.userId ?? 'authenticated') : null;
  const shownIds = useAppSelector(state => state.announcement.shownIds);
  const activeUserRef = useRef(authKey);
  const [result, setResult] = useState<{
    authKey: string;
    announcement: Announcement | null;
  } | null>(null);

  useEffect(() => {
    activeUserRef.current = authKey;
    if (!authKey) return;
    let cancelled = false;
    void fetchLatestAnnouncement()
      .then(announcement => {
        if (!cancelled) setResult({ authKey, announcement });
      })
      .catch((error: unknown) => log('fetch failed: %O', error));
    return () => {
      cancelled = true;
      activeUserRef.current = null;
    };
  }, [authKey]);

  const announcement = authKey && result?.authKey === authKey ? result.announcement : null;
  if (!announcement || shownIds.includes(announcement.id)) return null;

  const dismiss = () => {
    if (activeUserRef.current !== authKey) return;
    dispatch(markAnnouncementShown(announcement.id));
    setResult(null);
  };

  const cta = announcement.cta;
  return {
    id: `announcement:${announcement.id}`,
    severity: SEVERITY[announcement.severity],
    title: announcement.title,
    body: announcement.body,
    onDismiss: dismiss,
    ...(cta
      ? {
          actionLabel: cta.label,
          onAction: () => {
            // Retain the notice if the OS refuses the URL so it can be retried.
            void openUrl(cta.url)
              .then(dismiss)
              .catch((error: unknown) => log('open announcement link failed: %O', error));
          },
        }
      : {}),
  };
}
