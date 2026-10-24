import type { Thread } from '../../../types/thread';

function startOfLocalDay(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

/**
 * Reserved thread label that marks a conversation as pinned. Persisted by the
 * core like any other label (`openhuman.threads_update_labels`), so a pin
 * survives restarts and follows the workspace. It never makes a thread a task
 * (`isThreadVisibleInTab` keys tasks off other labels), so pinned threads stay
 * in the General tab.
 */
export const PINNED_THREAD_LABEL = 'pinned';
/** Reserved label used to keep archived conversations in the core thread store. */
export const ARCHIVED_THREAD_LABEL = 'ui:archived';
/** Prefix for user-assigned sidebar categories stored in the thread's labels. */
export const THREAD_CATEGORY_LABEL_PREFIX = 'category:';

type RecencyGroupKey =
  | 'pinned'
  | 'today'
  | 'yesterday'
  | 'previous7Days'
  | 'previous30Days'
  | 'older';

export type ThreadGroupKey = RecencyGroupKey | `category:${string}` | `project:${string}`;

export interface ThreadGroup {
  key: ThreadGroupKey;
  /** User-defined category or project folder name; recency groups use translations. */
  label?: string;
  threads: Thread[];
}

export function isThreadPinned(thread: Thread): boolean {
  return Boolean(thread.labels?.includes(PINNED_THREAD_LABEL));
}

export function isThreadArchived(thread: Thread): boolean {
  return Boolean(thread.labels?.includes(ARCHIVED_THREAD_LABEL));
}

export function threadCategory(thread: Thread): string | null {
  const raw = thread.labels?.find(label => label.startsWith(THREAD_CATEGORY_LABEL_PREFIX));
  const category = raw?.slice(THREAD_CATEGORY_LABEL_PREFIX.length).trim();
  return category ? category : null;
}

/** Windows directory spelling is case-insensitive; Unix spelling is not. */
function projectDirectoryKey(path: string): string {
  const windowsPath = /^(?:[a-z]:[\\/]|\\\\|\/\/)/i.test(path);
  const normalized = windowsPath ? path.replace(/\//g, '\\').toLowerCase() : path;
  return normalized.replace(/[\\/]+$/, '') || normalized;
}

/** Labels with the pin toggled; every other label is kept in order. */
export function labelsWithPin(labels: readonly string[] | undefined, pinned: boolean): string[] {
  const rest = (labels ?? []).filter(label => label !== PINNED_THREAD_LABEL);
  return pinned ? [...rest, PINNED_THREAD_LABEL] : rest;
}

export function labelsWithArchived(
  labels: readonly string[] | undefined,
  archived: boolean
): string[] {
  const rest = (labels ?? []).filter(label => label !== ARCHIVED_THREAD_LABEL);
  return archived ? [...rest, ARCHIVED_THREAD_LABEL] : rest;
}

export function labelsWithCategory(
  labels: readonly string[] | undefined,
  category: string | null
): string[] {
  const rest = (labels ?? []).filter(label => !label.startsWith(THREAD_CATEGORY_LABEL_PREFIX));
  const normalized = category?.trim().replace(/\s+/g, ' ');
  return normalized ? [...rest, `${THREAD_CATEGORY_LABEL_PREFIX}${normalized}`] : rest;
}

/** Case-insensitive title match; a blank query matches everything. */
export function threadMatchesQuery(title: string, query: string): boolean {
  const needle = query.trim().toLowerCase();
  return needle === '' || title.toLowerCase().includes(needle);
}

/** Which recency bucket a timestamp falls into, relative to `now`'s local day. */
export function recencyGroupFor(timestamp: string, now: Date): Exclude<RecencyGroupKey, 'pinned'> {
  const time = new Date(timestamp).getTime();
  // An unparseable timestamp sorts to the bottom rather than claiming "Today".
  if (Number.isNaN(time)) return 'older';
  const dayStart = (offset: number) =>
    startOfLocalDay(new Date(now.getFullYear(), now.getMonth(), now.getDate() - offset));
  if (time >= dayStart(0)) return 'today';
  if (time >= dayStart(1)) return 'yesterday';
  if (time >= dayStart(7)) return 'previous7Days';
  if (time >= dayStart(30)) return 'previous30Days';
  return 'older';
}

const GROUP_ORDER: RecencyGroupKey[] = [
  'pinned',
  'today',
  'yesterday',
  'previous7Days',
  'previous30Days',
  'older',
];

/**
 * Groups pinned and explicitly categorized/project threads before the remaining
 * recency buckets. Input order is preserved inside each section.
 */
export function groupThreads(
  threads: readonly Thread[],
  now: Date,
  isPinned: (thread: Thread) => boolean = isThreadPinned
): ThreadGroup[] {
  const buckets = new Map<ThreadGroupKey, ThreadGroup>();
  for (const thread of threads) {
    const category = threadCategory(thread);
    const projectPath = thread.actionDir?.trim();
    const project = projectPath ? folderBasename(projectPath) : null;
    let key: ThreadGroupKey;
    let label: string | undefined;
    if (isPinned(thread)) {
      key = 'pinned';
    } else if (category) {
      key = `category:${category.toLowerCase()}`;
      label = category;
    } else if (projectPath && project) {
      key = `project:${projectDirectoryKey(projectPath)}`;
      label = project;
    } else {
      key = recencyGroupFor(thread.lastMessageAt, now);
    }
    const bucket = buckets.get(key);
    if (bucket) bucket.threads.push(thread);
    else buckets.set(key, { key, ...(label ? { label } : {}), threads: [thread] });
  }
  const orderedDynamic = [...buckets.values()]
    .filter(group => group.key.startsWith('category:') || group.key.startsWith('project:'))
    .sort((a, b) => {
      const aType = a.key.startsWith('category:') ? 0 : 1;
      const bType = b.key.startsWith('category:') ? 0 : 1;
      return (
        aType - bType ||
        (a.label ?? '').localeCompare(b.label ?? '', undefined, { sensitivity: 'base' })
      );
    });
  const orderedRecency = GROUP_ORDER.flatMap(key => {
    const group = buckets.get(key);
    return group ? [group] : [];
  });
  const pinned = buckets.get('pinned');
  const recencyStart = pinned ? 1 : 0;
  return [...(pinned ? [pinned] : []), ...orderedDynamic, ...orderedRecency.slice(recencyStart)];
}

/** Last path segment of a folder, for a compact tooltip. */
export function folderBasename(path: string): string {
  const trimmed = path.replace(/[\\/]+$/, '');
  const parts = trimmed.split(/[\\/]/);
  return parts[parts.length - 1] || trimmed || path;
}
