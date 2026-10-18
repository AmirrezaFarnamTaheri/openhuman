import { describe, expect, it } from 'vitest';

import {
  getDefaultEnabledTools,
  getEnabledRustToolNames,
  normalizeEnabledToolList,
  TOOL_CATALOG,
} from './toolDefinitions';

describe('normalizeEnabledToolList', () => {
  it('maps structural inspection and mutation aliases to the existing file controls', () => {
    const readers = ['ast_outline', 'ast_symbol', 'chunk_search', 'chunk_inspect'];
    const writers = ['ast_patch', 'chunk_apply'];
    const enabledReaders = getEnabledRustToolNames(['file_read']);
    const enabledWriters = getEnabledRustToolNames(['file_write']);
    for (const name of readers) {
      expect(enabledReaders).toContain(name);
      expect(enabledWriters).not.toContain(name);
      expect(normalizeEnabledToolList([name])).toEqual(['file_read']);
    }
    for (const name of writers) {
      expect(enabledWriters).toContain(name);
      expect(enabledReaders).not.toContain(name);
      expect(normalizeEnabledToolList([name])).toEqual(['file_write']);
    }
  });

  it('round-trips persisted structural aliases without duplicate or new toggles', () => {
    const restored = normalizeEnabledToolList([
      'ast_outline',
      'chunk_inspect',
      'file_read',
      'ast_patch',
      'chunk_apply',
    ]);
    expect(restored).toEqual(['file_read', 'file_write']);
    expect(normalizeEnabledToolList(getEnabledRustToolNames(restored))).toEqual(restored);
    expect(TOOL_CATALOG.map(tool => tool.id)).not.toContain('ast_patch');
  });

  it('converts a Rust tool name to its UI toggle ID', () => {
    // "web_search_tool" is the Rust name for the "web_search" UI toggle.
    // This was the root cause of #2742: handleSave persisted Rust names,
    // the read path checked UI IDs, and the mismatch caused the toggle to
    // always read back as OFF.
    expect(normalizeEnabledToolList(['web_search_tool'])).toEqual(['web_search']);
  });

  it('passes through an entry that is already a UI toggle ID', () => {
    expect(normalizeEnabledToolList(['shell'])).toEqual(['shell']);
  });

  it('handles a mixed list of UI IDs and Rust names', () => {
    const result = normalizeEnabledToolList(['shell', 'web_search_tool', 'file_read']);
    expect(result).toContain('shell');
    expect(result).toContain('web_search');
    expect(result).toContain('file_read');
    expect(result).toHaveLength(3);
  });

  it('deduplicates when multiple Rust names map to the same UI toggle ID', () => {
    // "cron_add", "cron_list", and "cron_remove" all belong to the "cron" toggle.
    const result = normalizeEnabledToolList(['cron_add', 'cron_list', 'cron_remove']);
    expect(result).toEqual(['cron']);
  });

  it('maps the retired memory_store / memory_recall / memory_forget toggles to memory', () => {
    expect(normalizeEnabledToolList(['memory_store', 'memory_recall', 'memory_forget'])).toEqual([
      'memory',
    ]);
  });

  it('exposes one memory toggle backed by the single memory tool', () => {
    const memory = TOOL_CATALOG.filter(tool => tool.category === 'Memory');
    expect(memory.map(tool => tool.id)).toEqual(['memory']);
    expect(memory[0].rustToolNames).toEqual(['memory']);
    expect(TOOL_CATALOG.flatMap(tool => tool.rustToolNames)).not.toContain('update_memory_md');
  });

  it('drops unknown entries', () => {
    const result = normalizeEnabledToolList(['shell', 'totally_unknown_tool']);
    expect(result).toEqual(['shell']);
  });

  it('returns empty array for empty input', () => {
    expect(normalizeEnabledToolList([])).toEqual([]);
  });

  it('round-trips with getEnabledRustToolNames for all default enabled tools', () => {
    const defaults = getDefaultEnabledTools();
    const rustNames = getEnabledRustToolNames(defaults);
    const normalized = normalizeEnabledToolList(rustNames);
    // Every default UI ID must survive the round-trip.
    for (const id of defaults) {
      expect(normalized).toContain(id);
    }
  });

  it('normalises a realistic persisted list that would cause the #2742 revert', () => {
    // Simulate what handleSave writes: Rust names for all tools that were ON.
    const persistedRustNames = getEnabledRustToolNames([
      'shell',
      'web_search',
      'http_request',
      'cron',
    ]);
    const normalized = normalizeEnabledToolList(persistedRustNames);
    expect(normalized).toContain('shell');
    expect(normalized).toContain('web_search');
    expect(normalized).toContain('http_request');
    expect(normalized).toContain('cron');
  });

  it('covers every entry in TOOL_CATALOG — all rustToolNames reverse-map correctly', () => {
    for (const tool of TOOL_CATALOG) {
      for (const rustName of tool.rustToolNames) {
        const result = normalizeEnabledToolList([rustName]);
        expect(result).toEqual([tool.id]);
      }
    }
  });
});
