import importlib.util
import pathlib
import unittest

MODULE_PATH=pathlib.Path(__file__).resolve().parents[1]/'classify-peer-census.py'
spec=importlib.util.spec_from_file_location('peer_classify',MODULE_PATH)
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
class ClassificationTests(unittest.TestCase):
    def entry(self,path,kind='source'):
        return {'path':path,'kind':kind,'sha256':'00','symbols':['name']}
    def test_interception_gets_security_gate_before_routing(self):
        x=mod.classify('zero-gravity',self.entry('src/mitm/proxy.rs'))
        self.assertEqual(x['action'],'defer-privileged-transport')
    def test_account_oauth_is_native_host_owned(self):
        x=mod.classify('switchai',self.entry('src-tauri/src/providers/gemini/oauth.rs'))
        self.assertEqual(x['action'],'native-credential-owner-only')
    def test_models_become_unverified_declarations(self):
        x=mod.classify('antigravity-shield',self.entry('src-tauri/resources/model_specs.json'))
        self.assertEqual(x['owner'],'catalog')
    def test_unmapped_source_is_not_claimed_integrated(self):
        x=mod.classify('shield',self.entry('src/util/something.rs'))
        self.assertEqual(x['action'],'inspect-code-semantic-gap')
    def test_binary_images_are_not_implicitly_copied(self):
        x=mod.classify('shield',self.entry('public/image.png','binary'))
        self.assertEqual(x['action'],'reference-asset')

if __name__=='__main__':unittest.main()
