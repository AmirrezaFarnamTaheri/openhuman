import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const connections = readFileSync(new URL('../../app/src/pages/Skills.tsx', import.meta.url), 'utf8');
const memory = readFileSync(new URL('../../app/src/pages/Memory.tsx', import.meta.url), 'utf8');

test('Connections keeps advanced memory, MCP and Skills managers out of the default static chunk', () => {
  for (const component of ['Memory', 'McpServersPage', 'SkillsPage']) {
    assert.doesNotMatch(connections, new RegExp('^import ' + component + ' from', 'm'));
    assert.match(connections, new RegExp('const ' + component + ' = lazy\\(\\(\\) => import\\('));
  }
  assert.match(connections, /import\('\.\/Memory'\)/);
  assert.match(connections, /import\('\.\.\/components\/channels\/mcp\/McpServersPage'\)/);
  assert.match(connections, /import\('\.\.\/components\/skills\/SkillsPage'\)/);
});

test('lazy managers always provide a loading state instead of leaving an empty panel', () => {
  const count = (connections.match(/<Suspense fallback=/g) ?? []).length;
  assert.ok(count >= 3, 'Memory, MCP and Skills each need a suspense loading fallback');
});

test('memory administration screens are not eagerly parsed by Explorer', () => {
  for (const component of ['MemoryMigrationTab', 'MemoryBackgroundTab', 'MemorySettingsTab']) {
    assert.doesNotMatch(memory, new RegExp('^import ' + component + ' from', 'm'));
    assert.match(memory, new RegExp('const ' + component + ' = lazy\\(\\(\\) => import\\('));
  }
  assert.match(memory, /<Suspense fallback=/);
});

test('Connections does not parse provider management panels before they are selected', () => {
  for (const component of [
    'ComposioPanel', 'ComputerPanel', 'EmbeddingsPanel', 'LiveVoicePanel',
    'LlmConnectionsPanel', 'SearchPanel', 'ToolsPanel', 'UsagePanel',
    'VoiceConnectionsPanel', 'WalletPanel',
  ]) {
    assert.doesNotMatch(connections, new RegExp('^import ' + component + '[ ,]','m'));
    assert.match(connections, new RegExp('const ' + component + ' = lazy\\(\\(\\) => import\\('));
  }
});
