import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {buildCatalog} from '../catalog.mjs';
const data=JSON.parse(readFileSync(new URL('../peer-declared-models.json',import.meta.url),'utf8'));
test('every model declaration includes a concrete code source',()=>{
  assert.equal(data.schema_version,1);
  assert.ok(data.records.length>30);
  for(const r of data.records){assert.ok(r.id);assert.equal(r.provider,'antigravity');assert.ok(r.source.includes(':'))}
});
test('declared legacy, variant and image names remain unverified without a live gateway',()=>{
  const rows=buildCatalog(data.records,[]);
  assert.ok(rows.some(r=>r.id.includes('image')));
  assert.ok(rows.some(r=>r.id.includes('gemini')));
  assert.ok(rows.every(r=>r.availability==='unverified'&&r.pricing.known===false));
});
test('the same model can have several peer sources without duplicate model choices',()=>{
  const rows=buildCatalog(data.records,[]);
  const keys=new Set(rows.map(r=>`${r.provider}/${r.id}`));
  assert.equal(keys.size,rows.length);
  assert.ok(rows.some(r=>r.sources.length>1));
});
