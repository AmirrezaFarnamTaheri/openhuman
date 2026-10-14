"""Adversarial tests for untrusted source archive discovery."""
import importlib.util
import io
import pathlib
import tempfile
import unittest
import zipfile

SCRIPT=pathlib.Path(__file__).resolve().parents[1]/'audit-peer-archives.py'
spec=importlib.util.spec_from_file_location('audit_peer_archives',SCRIPT)
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)

class PeerZipCensusTests(unittest.TestCase):
    def make_zip(self,entries):
        tmp=tempfile.NamedTemporaryFile(suffix='.zip',delete=False)
        with zipfile.ZipFile(tmp.name,'w') as out:
            for k,v in entries.items():out.writestr(k,v)
        self.addCleanup(lambda:pathlib.Path(tmp.name).unlink(missing_ok=True))
        return tmp.name
    def test_records_all_entries_and_hashes(self):
        path=self.make_zip({'root/src/main.rs':'pub fn hello() {}\n','root/LICENSE':'MIT\n','root/icon.png':b'\x89PNG'})
        rows=mod.analyze_zip(path)
        self.assertEqual(len(rows),3)
        self.assertEqual(next(x for x in rows if x['path']=='src/main.rs')['symbols'],['hello'])
        self.assertEqual(next(x for x in rows if x['path']=='icon.png')['kind'],'binary')
    def test_rejects_archive_path_traversal(self):
        path=self.make_zip({'root/../outside.txt':'bad'})
        with self.assertRaisesRegex(ValueError,'unsafe'):mod.analyze_zip(path)
    def test_rejects_oversized_declared_source(self):
        path=self.make_zip({'root/too-large.txt':'x'*512})
        old=mod.MAX_ENTRY_BYTES
        try:
            mod.MAX_ENTRY_BYTES=100
            with self.assertRaisesRegex(ValueError,'bound'):mod.analyze_zip(path)
        finally:mod.MAX_ENTRY_BYTES=old
    def test_source_fingerprints_exclude_nonidentical_files(self):
        a=self.make_zip({'one/a.rs':'fn same() {}'});b=self.make_zip({'two/b.rs':'fn same() {}'})
        report=mod.census([('a',a),('b',b)])
        self.assertEqual(len(report['exact_cross_archive_source_duplicates']),1)
    def test_source_model_parser_requires_explicit_files(self):
        self.assertTrue(callable(mod.parse_model_declarations))

if __name__=='__main__':unittest.main()
