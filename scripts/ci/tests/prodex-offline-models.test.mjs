import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {enrichLiveProdexModels} from '../prodex-catalog-overlay.mjs';

const filename = fileURLToPath(import.meta.url);
const snapshot = JSON.parse(fs.readFileSync(path.join(path.dirname(filename), '..', 'prodex-offline-models.json'), 'utf8'));

test('snapshot has a stable provenance and all 82 Prodex models without duplicate provider/id', () => {
 assert.equal(snapshot.schema_version, 1);
 assert.match(snapshot.source_sha256,/^[a-f0-9]{64}$/);
 assert.equal(snapshot.models.length,snapshot.model_count);
 assert.equal(snapshot.model_count,82);
 const ids=snapshot.models.map(x=>`${x.provider}:${x.id}`);
 assert.equal(new Set(ids).size,ids.length);
 assert.equal(new Set(snapshot.models.map(x=>x.provider)).size,7);
});

test('offline catalog never makes pricing or unavailable models appear live', () => {
 assert.ok(snapshot.models.every(x=>x.pricing_known===false));
 assert.deepEqual(enrichLiveProdexModels([],snapshot.models),[]);
 const live=[{id:'gpt-5.4',provider:'openai'}];
 const result=enrichLiveProdexModels(live,snapshot.models);
 assert.equal(result.length,1);
 assert.equal(result[0].catalogStatus,'matched');
 assert.equal(result[0].pricing.known,false);
 assert.equal(result[0].contextWindowVerified,false);
});
