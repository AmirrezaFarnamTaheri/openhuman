import {
  ThreadListItem,
  ThreadListNew,
  ThreadListRoot,
} from '@/components/assistant-ui/thread-list';
import {
  AssistantRuntimeProvider,
  ThreadListItemMorePrimitive,
  ThreadListPrimitive,
  type ThreadMessageLike,
  useAuiState,
  useExternalStoreRuntime,
} from '@assistant-ui/react';
import { ArchiveIcon, ArchiveRestoreIcon, ChevronDownIcon, PinIcon } from 'lucide-react';
import { createContext, useContext, useMemo, useState } from 'react';

import Button from '../../../components/ui/Button';
import {
  CollapsibleContent,
  CollapsibleRoot,
  CollapsibleTrigger,
} from '../../../components/ui/Collapsible';
import { ModalShell } from '../../../components/ui/ModalShell';
import TextField from '../../../components/ui/TextField';
import { useT } from '../../../lib/i18n/I18nContext';
import type { Thread } from '../../../types/thread';
import { groupThreads, isThreadArchived, isThreadPinned, threadCategory } from './groupThreads';

const GROUP_LABEL_KEYS: Record<string, string> = {
  pinned: 'chat.sidebar.group.pinned',
  today: 'chat.sidebar.group.today',
  yesterday: 'chat.sidebar.group.yesterday',
  previous7Days: 'chat.sidebar.group.previous7Days',
  previous30Days: 'chat.sidebar.group.previous30Days',
  older: 'chat.sidebar.group.older',
};

interface ThreadListProps {
  threads: Thread[];
  /** Persisted descendants; rendered only beneath their parent, never as roots. */
  childThreads?: Thread[];
  selectedThreadId: string | null;
  onCreateThread: () => void | Promise<void>;
  onSelectThread: (threadId: string) => void;
  resolveTitle: (threadId: string) => string;
  isThreadRunning?: (threadId: string) => boolean;
  unreadThreadIds?: ReadonlySet<string>;
  isPinned?: (thread: Thread) => boolean;
  onTogglePin?: (thread: Thread, pinned: boolean) => void;
  onArchiveThread?: (thread: Thread, archived: boolean) => void | Promise<void>;
  onSetThreadCategory?: (thread: Thread, category: string | null) => void | Promise<void>;
  onRequestDelete: (thread: Thread) => void;
  onRenameThread: (threadId: string, title: string) => Promise<void>;
  collapsedGroups?: readonly string[];
  onToggleGroup?: (group: string) => void;
}

interface ThreadListHostContextValue extends ThreadListProps {
  openCategory: (thread: Thread) => void;
  reportActionError: (error: unknown) => void;
}

const ThreadListHostContext = createContext<ThreadListHostContextValue | null>(null);
const EMPTY_MESSAGES: readonly ThreadMessageLike[] = [];

/** Registry row plus OpenHuman's working folder, unread indicator and pin action. */
function ThreadRow() {
  const host = useContext(ThreadListHostContext)!;
  const { t } = useT();
  const id = useAuiState(s => s.threadListItem.id);
  const thread =
    host.threads.find(item => item.id === id) ?? host.childThreads?.find(item => item.id === id);
  if (!thread) return null;
  const running = Boolean(host.isThreadRunning?.(id));
  const pinned = (host.isPinned ?? isThreadPinned)(thread);
  const archived = isThreadArchived(thread);
  return (
    <ThreadListItem
      className="flex-none"
      running={running}
      showArchive={false}
      triggerProps={
        {
          'data-testid': `thread-row-${id}`,
          'data-analytics-id': 'chat-sidebar-thread-row',
          // Re-selecting the active thread still synchronizes OpenHuman's route.
          onClick: () => {
            if (host.selectedThreadId === id) host.onSelectThread(id);
          },
          title: thread.actionDir
            ? t('chat.sidebar.workingFolder').replace('{folder}', thread.actionDir)
            : undefined,
        } as React.ComponentPropsWithoutRef<'button'>
      }
      trailing={
        !running && host.unreadThreadIds?.has(id) ? (
          <span
            data-testid={`thread-unread-${id}`}
            role="img"
            aria-label={t('chat.sidebar.unread')}
            className="bg-primary size-1.5 shrink-0 rounded-full"
          />
        ) : null
      }
      menuExtras={
        <>
          {host.onTogglePin && (
            <ThreadListItemMorePrimitive.Item
              data-testid={`thread-pin-${id}`}
              onSelect={() => host.onTogglePin?.(thread, !pinned)}
              className="hover:bg-accent focus:bg-accent flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm outline-none">
              <PinIcon aria-hidden className="size-4" />
              {t(pinned ? 'chat.sidebar.unpinThread' : 'chat.sidebar.pinThread')}
            </ThreadListItemMorePrimitive.Item>
          )}
          {host.onSetThreadCategory && (
            <ThreadListItemMorePrimitive.Item
              data-testid={`thread-category-${id}`}
              onSelect={() => host.openCategory(thread)}
              className="hover:bg-accent focus:bg-accent flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm outline-none">
              <span aria-hidden className="size-4 text-center">
                #
              </span>
              {t('settings.costDashboard.category')}
            </ThreadListItemMorePrimitive.Item>
          )}
          {host.onArchiveThread && (
            <ThreadListItemMorePrimitive.Item
              data-testid={`thread-archive-${id}`}
              onSelect={() => {
                void Promise.resolve(host.onArchiveThread?.(thread, !archived)).catch(
                  host.reportActionError
                );
              }}
              className="hover:bg-accent focus:bg-accent flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm outline-none">
              {archived ? (
                <ArchiveRestoreIcon aria-hidden className="size-4" />
              ) : (
                <ArchiveIcon aria-hidden className="size-4" />
              )}
              {archived ? t('chat.sidebar.restoreThread') : t('assistantUi.threadList.archive')}
            </ThreadListItemMorePrimitive.Item>
          )}
        </>
      }
    />
  );
}

/** The core owns threads; assistant-ui owns selection and row actions. */
export function ThreadList(props: ThreadListProps) {
  const { threads, resolveTitle } = props;
  const [categoryThread, setCategoryThread] = useState<Thread | null>(null);
  const [threadActionError, setThreadActionError] = useState('');
  const adapterThreads = useMemo(
    () =>
      [...threads, ...(props.childThreads ?? [])]
        .filter((thread, index, all) => all.findIndex(item => item.id === thread.id) === index)
        .map(thread => ({
          id: thread.id,
          remoteId: thread.id,
          status: 'regular' as const,
          title: resolveTitle(thread.id),
        })),
    [threads, props.childThreads, resolveTitle]
  );
  const runtime = useExternalStoreRuntime({
    messages: EMPTY_MESSAGES,
    convertMessage: message => message,
    isRunning: false,
    onNew: async () => {},
    adapters: {
      threadList: {
        threads: adapterThreads,
        threadId: props.selectedThreadId ?? undefined,
        onSwitchToNewThread: props.onCreateThread,
        onSwitchToThread: props.onSelectThread,
        onRename: props.onRenameThread,
        onDelete: id => {
          const thread =
            props.threads.find(item => item.id === id) ??
            props.childThreads?.find(item => item.id === id);
          if (thread) props.onRequestDelete(thread);
        },
      },
    },
  });
  const hostValue: ThreadListHostContextValue = {
    ...props,
    openCategory: thread => {
      setThreadActionError('');
      setCategoryThread(thread);
    },
    reportActionError: error =>
      setThreadActionError(error instanceof Error ? error.message : String(error)),
  };
  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <ThreadListHostContext.Provider value={hostValue}>
        <ThreadListView />
        <ThreadCategoryDialog
          key={categoryThread?.id ?? 'closed'}
          thread={categoryThread}
          threads={threads}
          onOpenChange={open => {
            if (!open) setCategoryThread(null);
          }}
          onSave={async category => {
            if (!categoryThread || !props.onSetThreadCategory) return;
            await props.onSetThreadCategory(categoryThread, category);
            setCategoryThread(current => (current?.id === categoryThread.id ? null : current));
          }}
        />
        {threadActionError && (
          <p role="alert" className="px-3 pb-2 text-xs text-destructive">
            {threadActionError}
          </p>
        )}
      </ThreadListHostContext.Provider>
    </AssistantRuntimeProvider>
  );
}

function hasArchivedDescendant(
  id: string,
  children: readonly Thread[],
  ancestors: readonly string[] = []
): boolean {
  if (ancestors.length >= 32 || ancestors.includes(id)) return false;
  return children.some(
    child =>
      child.parentThreadId === id &&
      (isThreadArchived(child) || hasArchivedDescendant(child.id, children, [...ancestors, id]))
  );
}

/** A separate disclosure keeps delegated conversations out of the root list. */
function ThreadTreeRow({
  thread,
  threadIds,
  showArchived,
  archivedBranch = false,
  ancestors = [],
}: {
  thread: Thread;
  threadIds: readonly string[];
  showArchived: boolean;
  archivedBranch?: boolean;
  ancestors?: readonly string[];
}) {
  const host = useContext(ThreadListHostContext)!;
  const { t } = useT();
  const children =
    ancestors.length >= 32
      ? []
      : (host.childThreads ?? []).filter(
          child =>
            child.parentThreadId === thread.id &&
            child.id !== thread.id &&
            !ancestors.includes(child.id) &&
            threadIds.includes(child.id) &&
            (showArchived
              ? archivedBranch ||
                isThreadArchived(thread) ||
                isThreadArchived(child) ||
                hasArchivedDescendant(child.id, host.childThreads ?? [])
              : !isThreadArchived(child))
        );
  return (
    <>
      <ThreadListPrimitive.ItemByIndex
        index={threadIds.indexOf(thread.id)}
        components={{ ThreadListItem: ThreadRow }}
      />
      {children.length > 0 && (
        <CollapsibleRoot className="ms-3 border-s border-line ps-2">
          <CollapsibleTrigger size="sm" className="rounded-md px-2 py-1 text-xs text-content-muted">
            <span className="flex-1">{t('conversations.agentTaskInsights.subagentsHeading')}</span>
            <span className="tabular-nums">{children.length}</span>
            <ChevronDownIcon
              aria-hidden
              className="size-3.5 group-data-[state=closed]:-rotate-90"
            />
          </CollapsibleTrigger>
          <CollapsibleContent className="px-0 pb-0">
            {children.map(child => (
              <ThreadTreeRow
                key={child.id}
                thread={child}
                threadIds={threadIds}
                showArchived={showArchived}
                archivedBranch={archivedBranch || isThreadArchived(thread)}
                ancestors={[...ancestors, thread.id]}
              />
            ))}
          </CollapsibleContent>
        </CollapsibleRoot>
      )}
    </>
  );
}

/** Render only ids already admitted by the runtime, including during async list updates. */
function ThreadListView() {
  const props = useContext(ThreadListHostContext)!;
  const { t } = useT();
  const [showArchived, setShowArchived] = useState(false);
  const [localCollapsed, setLocalCollapsed] = useState<string[]>([]);
  const collapsed = props.collapsedGroups ?? localCollapsed;
  const toggleGroup = (key: string) => {
    if (props.onToggleGroup) props.onToggleGroup(key);
    else
      setLocalCollapsed(previous =>
        previous.includes(key) ? previous.filter(item => item !== key) : [...previous, key]
      );
  };
  const threadIds = useAuiState(s => s.threads.threadIds);
  const archivedCount = [...props.threads, ...(props.childThreads ?? [])].filter(
    isThreadArchived
  ).length;
  const visibleThreads = props.threads.filter(thread =>
    showArchived
      ? isThreadArchived(thread) || hasArchivedDescendant(thread.id, props.childThreads ?? [])
      : !isThreadArchived(thread)
  );
  const visibleIds = new Set(visibleThreads.map(thread => thread.id));
  const filtered = threadIds.flatMap(id => {
    const thread = props.threads.find(item => item.id === id);
    return thread && visibleIds.has(id) ? [thread] : [];
  });
  const groups = groupThreads(filtered, new Date(), props.isPinned ?? isThreadPinned);
  return (
    <ThreadListRoot className="h-full min-h-0">
      <div className="flex-none px-2 pb-2">
        <ThreadListNew
          onClick={() => setShowArchived(false)}
          data-testid="new-thread-button"
          data-analytics-id="chat-sidebar-new-thread"
          className="w-full"
          title={t('chat.newThreadShortcut')}
          label={t('chat.newConversation')}
        />
        <Button
          type="button"
          variant="tertiary"
          size="sm"
          className="mt-1 w-full justify-start text-content-muted"
          aria-pressed={showArchived}
          data-testid="thread-archive-view-toggle"
          onClick={() => setShowArchived(value => !value)}>
          <ArchiveIcon aria-hidden className="size-4" />
          {t('chat.sidebar.archived')}
          {archivedCount > 0 && <span className="ms-auto text-xs">{archivedCount}</span>}
        </Button>
      </div>
      <div
        data-slot="aui_thread-list-items"
        className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-3 [scrollbar-gutter:stable_both-edges] [scrollbar-width:thin]">
        {groups.map(group => (
          <section
            key={group.key}
            data-testid={`thread-group-${group.key}`}
            aria-label={group.label ?? t(GROUP_LABEL_KEYS[group.key] ?? 'chat.sidebar.group.older')}
            className="flex flex-col gap-0.5">
            <CollapsibleRoot
              open={!collapsed.includes(group.key)}
              onOpenChange={() => toggleGroup(group.key)}>
              <h3 className="pt-2">
                <CollapsibleTrigger
                  size="sm"
                  title={
                    group.key.startsWith('project:')
                      ? (group.threads[0]?.actionDir ?? undefined)
                      : undefined
                  }
                  data-slot="aui_thread-list-group-label"
                  className="rounded-md px-2.5 py-1.5 text-xs text-content-muted">
                  <span className="min-w-0 flex-1 truncate">
                    {group.label ?? t(GROUP_LABEL_KEYS[group.key] ?? 'chat.sidebar.group.older')}
                  </span>
                  <span className="text-content-muted tabular-nums">{group.threads.length}</span>
                  <ChevronDownIcon
                    aria-hidden
                    className="size-3.5 shrink-0 transition-transform group-data-[state=closed]:-rotate-90"
                  />
                </CollapsibleTrigger>
              </h3>
              <CollapsibleContent className="px-0 pb-0">
                {group.threads.map(thread => (
                  <ThreadTreeRow
                    key={thread.id}
                    thread={thread}
                    threadIds={threadIds}
                    showArchived={showArchived}
                  />
                ))}
              </CollapsibleContent>
            </CollapsibleRoot>
          </section>
        ))}
        {groups.length === 0 && (
          <p className="text-muted-foreground px-2.5 py-4 text-center text-xs">
            {t('chat.noThreads')}
          </p>
        )}
      </div>
    </ThreadListRoot>
  );
}

function ThreadCategoryDialog({
  thread,
  threads,
  onOpenChange,
  onSave,
}: {
  thread: Thread | null;
  threads: Thread[];
  onOpenChange: (open: boolean) => void;
  onSave: (category: string | null) => Promise<void>;
}) {
  const { t } = useT();
  const [name, setName] = useState(() => (thread ? (threadCategory(thread) ?? '') : ''));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const categories = useMemo(
    () => [
      ...new Set(threads.map(threadCategory).filter((value): value is string => Boolean(value))),
    ],
    [threads]
  );

  const save = async (category: string | null) => {
    setBusy(true);
    setError('');
    try {
      await onSave(category);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  if (!thread) return null;

  return (
    <ModalShell
      title={t('settings.costDashboard.category')}
      titleId="thread-category-dialog-title"
      closePolicy={{ escape: !busy, backdrop: !busy, button: !busy }}
      onClose={() => onOpenChange(false)}
      contentClassName="space-y-3 px-5 py-4"
      footer={
        <div className="flex justify-end gap-2">
          <Button
            type="button"
            variant="tertiary"
            disabled={busy}
            onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          {threadCategory(thread) && (
            <Button
              type="button"
              variant="secondary"
              disabled={busy}
              onClick={() => void save(null)}>
              {t('common.clear')}
            </Button>
          )}
          <Button
            type="submit"
            form="thread-category-form"
            variant="primary"
            disabled={busy || !name.trim()}>
            {t('common.save')}
          </Button>
        </div>
      }>
      <form
        id="thread-category-form"
        className="space-y-1.5"
        onSubmit={event => {
          event.preventDefault();
          if (!busy && name.trim()) void save(name.trim().replace(/\s+/g, ' '));
        }}>
        <label htmlFor="thread-category-name" className="text-xs text-content-secondary">
          {t('common.name')}
        </label>
        <TextField
          id="thread-category-name"
          autoFocus
          maxLength={64}
          value={name}
          onChange={event => setName(event.target.value)}
          data-testid="thread-category-input"
        />
        {categories.length > 0 && (
          <div className="flex flex-wrap gap-1.5" aria-label={t('settings.costDashboard.category')}>
            {categories.map(category => (
              <Button
                key={category}
                type="button"
                variant="secondary"
                size="xs"
                onClick={() => setName(category)}>
                {category}
              </Button>
            ))}
          </div>
        )}
        {error && (
          <p role="alert" className="text-xs text-destructive">
            {error}
          </p>
        )}
      </form>
    </ModalShell>
  );
}
