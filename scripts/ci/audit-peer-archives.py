#!/usr/bin/env python3
"""Read-only, deterministic ZIP source census. No extracted content is executed."""
from __future__ import annotations
import argparse
import collections
import hashlib
import json
import re
import sys
import zipfile
from pathlib import PurePosixPath

MAX_ENTRIES = 20_000
MAX_TOTAL_BYTES = 250_000_000
MAX_ENTRY_BYTES = 20_000_000
SOURCE_EXT = {'.rs','.ts','.tsx','.js','.mjs','.py','.c','.json','.toml','.yaml','.yml','.css','.html'}
TEXT_EXT = SOURCE_EXT | {'.md','.txt','.sh','.ps1','.xml','.svg'}
TAG_KEYWORDS = {
    'accounts': ('account','credential','oauth','token','auth','secret','vault'),
    'models': ('model','variant','preset','alias','catalog','provider'),
    'quota': ('quota','limits','usage','consumption','rate_limit','rate-limit','predict'),
    'routing': ('proxy','router','route','retry','circuit','upstream','pool','backoff'),
    'streaming': ('stream','sse','response','completions','protocol','protobuf','grpc'),
    'images_media': ('image','video','audio','vision','multimodal'),
    'tools_ecosystem': ('mcp','tools','skill','integration','opencode','droid'),
    'desktop_ide': ('ide','platform','window','tauri','extension','tray','process','cache'),
    'ux': ('ui','view','component','dashboard','widget','chart','status','sidebar','modal','tooltip','accessibility'),
    'security': ('security','redact','ip_filter','firewall','csrf','fingerprint','mitm','intercept'),
    'reliability': ('refresh','recovery','atomic','sync','backup','scheduler','health','monitor'),
    'testing': ('test','spec','mock','fixture','e2e','adversarial','ci','workflow'),
}
RUST_SYMBOL = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:fn|struct|enum|trait|mod)\s+(\w+)',re.M)
JS_SYMBOL = re.compile(r'^\s*(?:export\s+)?(?:declare\s+)?(?:async\s+)?(?:function|class|interface|type|const|let)\s+(\w+)',re.M)
PY_SYMBOL = re.compile(r'^\s*(?:async\s+)?(?:def|class)\s+(\w+)',re.M)

def analyze_zip(path: str):
    out=[]
    with zipfile.ZipFile(path) as z:
        infos=z.infolist()
        if len(infos)>MAX_ENTRIES:raise ValueError('archive contains too many entries')
        total=0
        for info in infos:
            if info.is_dir():continue
            p=PurePosixPath(info.filename)
            if p.is_absolute() or '..' in p.parts or '\\' in info.filename or ':' in p.parts[0]:
                raise ValueError('unsafe archive path')
            if info.file_size>MAX_ENTRY_BYTES:raise ValueError('individual entry exceeds bound')
            total+=info.file_size
            if total>MAX_TOTAL_BYTES:raise ValueError('archive exceeds uncompressed limit')
            data=z.read(info)
            if len(data)!=info.file_size:raise ValueError('archive size mismatch')
            parts=p.parts
            rel='/'.join(parts[1:]) if len(parts)>1 else parts[0]
            suffix=p.suffix.lower()
            text=data.decode('utf-8',errors='replace') if suffix in TEXT_EXT else None
            sym=[]
            if suffix=='.rs':sym=RUST_SYMBOL.findall(text or '')
            elif suffix in {'.ts','.tsx','.js','.mjs'}:sym=JS_SYMBOL.findall(text or '')
            elif suffix=='.py':sym=PY_SYMBOL.findall(text or '')
            matches=sorted(tag for tag,words in TAG_KEYWORDS.items() if any(word in rel.lower() for word in words))
            out.append({'path':rel,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'kind':'source' if suffix in SOURCE_EXT else 'text' if suffix in TEXT_EXT else 'binary','lines':text.count('\n')+int(bool(text and not text.endswith('\n'))) if text is not None else None,'symbols':sym,'tags':matches})
    return sorted(out,key=lambda f:f['path'])

def census(archives):
    all_archives=[]; source_dupes=collections.defaultdict(list)
    for label,path in archives:
        files=analyze_zip(path)
        categories=collections.Counter(tag for f in files for tag in f['tags'])
        for f in files:
            if f['kind']=='source':source_dupes[f['sha256']].append(f'{label}:{f["path"]}')
        all_archives.append({'name':label,'files':files,'total_files':len(files),'source_files':sum(f['kind']=='source' for f in files),'tagged_file_counts':dict(sorted(categories.items()))})
    shared={digest:paths for digest,paths in source_dupes.items() if len(set(x.split(':')[0] for x in paths))>1}
    return {'schema_version':1,'archives':all_archives,'exact_cross_archive_source_duplicates':shared}

def render_summary(report):
    rows=['# Peer archive: complete file-level source census','','This manifest covers every file in the provided archives. Tags are *path-derived discovery aids*, not assertions that every function was manually validated. SHA-256 hashes are computed over original ZIP bytes.','', '| Archive | Files | Source/config files | Source domains discovered |','|---|---:|---:|---|']
    for a in report['archives']:
        rows.append(f"| `{a['name']}` | {a['total_files']} | {a['source_files']} | {', '.join(sorted(a['tagged_file_counts']))} |")
    rows+=['',f"Exact same-byte source files shared across peer archives: **{len(report['exact_cross_archive_source_duplicates'])}**",'', 'Full per-file paths, sizes, hashes, symbols and tagged domains are in `PEER_SOURCE_CENSUS.json`. Identical names do not imply interchangeable semantics.']
    return '\n'.join(rows)+'\n'

def parse_model_declarations(panel_dir,shield_dir,zero_dir):
    # Source declarations, not authenticated model availability.
    from pathlib import Path
    out=[]
    def add(id,source):
        if isinstance(id,str) and 0<len(id)<256 and re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._:/-]*',id):
            out.append({'id':id,'provider':'antigravity','source':source})
    model_ts=Path(shield_dir)/'src/config/modelConfig.ts'
    for id in re.findall(r"^    '([^']+)':\s*\{",model_ts.read_text(encoding='utf-8'),flags=re.M):add(id,'Antigravity-Shield:src/config/modelConfig.ts')
    specs=json.loads((Path(shield_dir)/'src-tauri/resources/model_specs.json').read_text(encoding='utf-8'))
    for id in specs.get('models',{}):add(id,'Antigravity-Shield:src-tauri/resources/model_specs.json')
    for m in re.findall(r'\bname:\s*"([^\"]+)"', (Path(zero_dir)/'src/api/models.rs').read_text(encoding='utf-8')):add(m,'zero-gravity:src/api/models.rs')
    strategy=json.loads((Path(panel_dir)/'src/shared/config/quota_strategy.json').read_text(encoding='utf-8'))
    for group in strategy.get('groups',[]):
        for model in group.get('models',[]):add(model.get('id'),'antigravity-panel:src/shared/config/quota_strategy.json')
    out=sorted({(r['id'],r['source']):r for r in out}.values(),key=lambda r:(r['id'],r['source']))
    return {'schema_version':1,'disclaimer':'Peer declared model names only; not live availability, capability, price or provider entitlement. No aliases activated automatically.','records':out}

def main(argv=None):
    parser=argparse.ArgumentParser()
    parser.add_argument('--archive',action='append',required=True,help='LABEL=/path/to/archive.zip')
    parser.add_argument('--json-output',required=True)
    parser.add_argument('--md-output',required=True)
    args=parser.parse_args(argv)
    archives=[]
    for spec in args.archive:
        label,sep,path=spec.partition('=')
        if not sep or not label or not path:parser.error('archive must be LABEL=PATH')
        archives.append((label,path))
    report=census(archives)
    from pathlib import Path
    Path(args.json_output).write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    Path(args.md_output).write_text(render_summary(report),encoding='utf-8')
    print('Archive census:',[(x['name'],x['total_files']) for x in report['archives']])

if __name__=='__main__':sys.exit(main())
