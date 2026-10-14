#!/usr/bin/env python3
"""Deterministic per-file integration triage. A classification is not semantic review."""
import argparse
import collections
import json
from pathlib import Path

# Ordered: security-sensitive modules must win before generic path tags.
RULES = [
    (lambda n,p,f: '/mitm/' in '/'+p+'/' or p.startswith('src/mitm/'), 'defer-privileged-transport', 'MITM/traffic interception requires separate threat model and explicit consent.', 'security'),
    (lambda n,p,f: 'fingerprint' in p.lower() or 'network_patcher' in p.lower(), 'defer-identity-modification', 'Per-account/device impersonation and binary patching require explicit compatibility and privacy review.', 'security'),
    (lambda n,p,f: 'oauth' in p.lower() or 'secret_store' in p.lower() or 'token_utils' in p.lower(), 'native-credential-owner-only', 'Reconcile with existing keychain and OAuth owner; never expose credentials to browser/CI.', 'host-auth'),
    (lambda n,p,f: 'state.vscdb' in p.lower() or 'brain_scanner' in p.lower() or 'cache.service' in p.lower(), 'native-recovery-review', 'Filesystem/IDE-state operation needs backups, safe path handling and explicit native host action.', 'host-native'),
    (lambda n,p,f: f['kind']=='binary', 'reference-asset', 'Inspect attribution and usage only if this concrete asset is needed; not copied.', 'presentation'),
    (lambda n,p,f: f['kind']=='text' and not p.endswith(('.js','.ts','.rs','.py')), 'reference-document', 'Source reference or build/license documentation, not executable feature.', 'documentation'),
    (lambda n,p,f: ('test' in p.lower() or '/tests/' in '/'+p+'/') and f['kind']=='source', 'evaluate-regression-test', 'Analyze test invariants for host-level parity without blindly copying tests.', 'quality'),
    (lambda n,p,f: 'modelconfig' in p.lower() or 'model_specs' in p.lower() or p.endswith('/models.rs') or 'quota_strategy.json' in p.lower(), 'model-declaration-candidate', 'Record IDs with source provenance; require live verification before UI claims.', 'catalog'),
    (lambda n,p,f: 'quota' in p.lower() or 'usage' in p.lower() or 'refresh' in p.lower(), 'quota-telemetry-candidate', 'Compare freshness, units, reset and retry semantics against the single host observation owner.', 'observability'),
    (lambda n,p,f: 'dashboard' in p.lower() or 'components' in p.lower() or 'webview' in p.lower() or p.startswith('ui/'), 'ui-adaptation-candidate', 'Reconcile UI framework and accessibility with actual OpenHuman desktop app.', 'ui'),
    (lambda n,p,f: 'proxy' in p.lower() or 'adapter' in p.lower() or 'api/' in p.lower() or 'route' in p.lower(), 'protocol-differential-candidate', 'Compare semantics with host and delegated gateway; do not create a second credential/router owner.', 'transport'),
    (lambda n,p,f: 'provider' in p.lower() or 'preset' in p.lower() or 'account' in p.lower(), 'provider-fleet-candidate', 'Compare native provider lifecycle and model/preset semantics with host owner.', 'fleet'),
    (lambda n,p,f: p.startswith('.github/') or p.startswith('scripts/'), 'build-quality-candidate', 'Reconcile build/workflow behavior and external side effects; no blind script execution.', 'quality'),
    (lambda n,p,f: f['kind']=='source', 'inspect-code-semantic-gap', 'Source code requires a target-aware behavioral diff; not automatically ported.', 'unmapped'),
    (lambda n,p,f: True, 'reference-other', 'Retain source provenance and evaluate only if needed.', 'unmapped')
]

def classify(name,file):
    path=file['path']
    for when,action,reason,owner in RULES:
        if when(name,path,file):
            return {'archive':name,'path':path,'kind':file['kind'],'sha256':file['sha256'],'action':action,'owner':owner,'reason':reason,'symbols_count':len(file['symbols'])}
    raise RuntimeError('unreachable')

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('census')
    parser.add_argument('output')
    args=parser.parse_args()
    report=json.loads(Path(args.census).read_text(encoding='utf-8'))
    classified=[classify(a['name'],f) for a in report['archives'] for f in a['files']]
    counts=collections.Counter(row['action'] for row in classified)
    output={'schema_version':1,'warning':'Machine-assisted per-file TRIAGE, not manual code review or implementation completion.','entries':classified,'action_counts':dict(sorted(counts.items()))}
    Path(args.output).write_text(json.dumps(output,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    print('Classified',len(classified),'files',dict(counts))

if __name__=='__main__':main()
