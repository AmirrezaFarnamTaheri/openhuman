import { describe, expect, it } from 'vitest';

import type { Thread } from '../../../types/thread';
import {
  ARCHIVED_THREAD_LABEL,
  folderBasename,
  groupThreads,
  isThreadArchived,
  isThreadPinned,
  labelsWithArchived,
  labelsWithCategory,
  labelsWithPin,
  PINNED_THREAD_LABEL,
  recencyGroupFor,
  THREAD_CATEGORY_LABEL_PREFIX,
  threadCategory,
  threadMatchesQuery,
} from './groupThreads';

const NOW = new Date(2026, 9, 6, 15, 0, 0);

function thread(id: string, lastMessageAt: Date, labels: string[] = []): Thread {
  return {
    id,
    title: id,
    chatId: null,
    isActive: true,
    messageCount: 1,
    lastMessageAt: lastMessageAt.toISOString(),
    createdAt: lastMessageAt.toISOString(),
    labels,
  };
}

const daysAgo = (days: number, hour = 12) =>
  new Date(NOW.getFullYear(), NOW.getMonth(), NOW.getDate() - days, hour);

describe('recencyGroupFor', () => {
  it('buckets by local calendar day', () => {
    expect(recencyGroupFor(daysAgo(0, 0).toISOString(), NOW)).toBe('today');
    expect(recencyGroupFor(daysAgo(1, 23).toISOString(), NOW)).toBe('yesterday');
    expect(recencyGroupFor(daysAgo(1, 0).toISOString(), NOW)).toBe('yesterday');
    expect(recencyGroupFor(daysAgo(5).toISOString(), NOW)).toBe('previous7Days');
    expect(recencyGroupFor(daysAgo(20).toISOString(), NOW)).toBe('previous30Days');
    expect(recencyGroupFor(daysAgo(90).toISOString(), NOW)).toBe('older');
  });

  it('sends unparseable timestamps to older', () => {
    expect(recencyGroupFor('not a date', NOW)).toBe('older');
  });
});

describe('groupThreads', () => {
  it('merges Windows directory spellings but preserves Unix directory case', () => {
    const groups = groupThreads(
      [
        { ...thread('win-a', NOW), actionDir: 'C:\\Work\\Repo\\' },
        { ...thread('win-b', NOW), actionDir: 'c:/work/repo' },
        { ...thread('unix-a', NOW), actionDir: '/work/Repo' },
        { ...thread('unix-b', NOW), actionDir: '/work/repo' },
      ],
      NOW
    );
    expect(groups).toHaveLength(3);
    expect(
      groups.find(group => group.key === 'project:c:\\work\\repo')?.threads.map(item => item.id)
    ).toEqual(['win-a', 'win-b']);
    expect(groups.map(group => group.key)).toEqual(
      expect.arrayContaining(['project:/work/Repo', 'project:/work/repo'])
    );
  });
  it('puts pinned threads first and keeps order within sections', () => {
    const groups = groupThreads(
      [
        thread('a', daysAgo(0)),
        thread('p', daysAgo(40), [PINNED_THREAD_LABEL]),
        thread('b', daysAgo(0, 9)),
        thread('c', daysAgo(3)),
      ],
      NOW
    );
    expect(groups.map(g => [g.key, g.threads.map(t => t.id)])).toEqual([
      ['pinned', ['p']],
      ['today', ['a', 'b']],
      ['previous7Days', ['c']],
    ]);
  });

  it('honours a custom pin predicate', () => {
    const groups = groupThreads([thread('a', daysAgo(0))], NOW, () => true);
    expect(groups).toEqual([{ key: 'pinned', threads: [expect.objectContaining({ id: 'a' })] }]);
  });

  it('groups custom categories before project directories and recency', () => {
    const firstProject = { ...thread('project-a', daysAgo(4)), actionDir: 'C:\\work\\alpha' };
    const secondProject = { ...thread('project-b', daysAgo(0)), actionDir: 'D:\\work\\beta' };
    const explicitCategory = thread('category-a', daysAgo(2), [
      `${THREAD_CATEGORY_LABEL_PREFIX}Research`,
    ]);
    const groups = groupThreads(
      [thread('recent', daysAgo(0)), secondProject, explicitCategory, firstProject],
      NOW
    );

    expect(
      groups.map(group => [group.key, group.label, group.threads.map(item => item.id)])
    ).toEqual([
      ['category:research', 'Research', ['category-a']],
      ['project:c:\\work\\alpha', 'alpha', ['project-a']],
      ['project:d:\\work\\beta', 'beta', ['project-b']],
      ['today', undefined, ['recent']],
    ]);
  });

  it('keeps pins ahead of categories and merges category names case-insensitively', () => {
    const groups = groupThreads(
      [
        thread('category-a', daysAgo(0), [`${THREAD_CATEGORY_LABEL_PREFIX}Research`]),
        thread('pinned', daysAgo(0), [PINNED_THREAD_LABEL]),
        thread('category-b', daysAgo(1), [`${THREAD_CATEGORY_LABEL_PREFIX}research`]),
      ],
      NOW
    );

    expect(groups.map(group => [group.key, group.threads.map(item => item.id)])).toEqual([
      ['pinned', ['pinned']],
      ['category:research', ['category-a', 'category-b']],
    ]);
  });

  it('returns no sections for no threads', () => {
    expect(groupThreads([], NOW)).toEqual([]);
  });
});

describe('pin labels', () => {
  it('toggles only the reserved label', () => {
    expect(labelsWithPin(['general'], true)).toEqual(['general', PINNED_THREAD_LABEL]);
    expect(labelsWithPin(['general', PINNED_THREAD_LABEL], true)).toEqual([
      'general',
      PINNED_THREAD_LABEL,
    ]);
    expect(labelsWithPin([PINNED_THREAD_LABEL, 'general'], false)).toEqual(['general']);
    expect(labelsWithPin(undefined, false)).toEqual([]);
    expect(isThreadPinned(thread('x', NOW, [PINNED_THREAD_LABEL]))).toBe(true);
    expect(isThreadPinned(thread('y', NOW))).toBe(false);
  });
});

describe('archive and category labels', () => {
  it('toggles archive state without removing unrelated labels', () => {
    expect(labelsWithArchived(['general', PINNED_THREAD_LABEL], true)).toEqual([
      'general',
      PINNED_THREAD_LABEL,
      ARCHIVED_THREAD_LABEL,
    ]);
    expect(labelsWithArchived(['general', ARCHIVED_THREAD_LABEL], false)).toEqual(['general']);
    expect(isThreadArchived(thread('archived', NOW, [ARCHIVED_THREAD_LABEL]))).toBe(true);
    expect(isThreadArchived(thread('active', NOW))).toBe(false);
  });

  it('replaces or clears only the category label and normalizes whitespace', () => {
    const labels = ['general', `${THREAD_CATEGORY_LABEL_PREFIX}Old`, PINNED_THREAD_LABEL];
    expect(labelsWithCategory(labels, '  New   Category ')).toEqual([
      'general',
      PINNED_THREAD_LABEL,
      `${THREAD_CATEGORY_LABEL_PREFIX}New Category`,
    ]);
    expect(labelsWithCategory(labels, null)).toEqual(['general', PINNED_THREAD_LABEL]);
    expect(threadCategory(thread('categorized', NOW, labels))).toBe('Old');
  });
});

describe('threadMatchesQuery', () => {
  it('matches case-insensitively and treats blank as all', () => {
    expect(threadMatchesQuery('Fix Gmail OAuth', 'gmail')).toBe(true);
    expect(threadMatchesQuery('Fix Gmail OAuth', '  ')).toBe(true);
    expect(threadMatchesQuery('Fix Gmail OAuth', 'slack')).toBe(false);
  });
});

describe('folderBasename', () => {
  it('returns the last segment on either separator', () => {
    expect(folderBasename('/home/me/projects/site/')).toBe('site');
    expect(folderBasename('C:\\Users\\me\\repo')).toBe('repo');
    expect(folderBasename('/')).toBe('/');
  });
});
