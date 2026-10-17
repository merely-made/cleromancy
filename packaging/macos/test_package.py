"""Focused rejection tests; no native app is launched."""
import importlib.util
import json
from pathlib import Path
import plistlib
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('macos_package', Path(__file__).with_name('package.py'))
package = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(package)


class BundleIntegrityTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.app = Path(self.temporary.name) / 'Candidate.app'
        (self.app / 'Contents/MacOS').mkdir(parents=True)
        resources = self.app / 'Contents/Resources'
        resources.mkdir()
        (self.app / 'Contents/MacOS/Cleromancy').write_bytes(package.LAUNCHER)
        self.payload = resources / 'notice.txt'
        self.payload.write_text('Public package notice\n')
        (self.app / 'Contents/Info.plist').write_bytes(plistlib.dumps({
            'CFBundleExecutable': 'Cleromancy', 'CFBundlePackageType': 'APPL'}))
        manifest = {'binaries': {}, 'files': {str(p.relative_to(self.app)): {
            'sha256': package.digest(p), 'mode': p.stat().st_mode & 0o777}
            for p in self.app.rglob('*') if p.is_file()}}
        (resources / 'package-manifest.json').write_text(json.dumps(manifest))

    def test_clean_allowlist_passes_with_native_tools_stubbed(self):
        with patch.object(package, 'run', return_value=''):
            package.verify(self.app)

    def test_changed_bytes_are_rejected(self):
        self.payload.write_text('Changed notice\n')
        with self.assertRaisesRegex(ValueError, 'hash/mode mismatch'):
            package.verify(self.app)

    def test_executable_mode_loss_is_rejected(self):
        self.payload.chmod(0o755)
        with self.assertRaisesRegex(ValueError, 'hash/mode mismatch'):
            package.verify(self.app)

    def test_unlisted_store_is_rejected(self):
        (self.payload.parent / 'unexpected-store.redb').write_bytes(b'public synthetic test')
        with self.assertRaisesRegex(ValueError, 'allowlist mismatch'):
            package.verify(self.app)

    def test_symlink_is_rejected(self):
        (self.payload.parent / 'link').symlink_to('notice.txt')
        with self.assertRaisesRegex(ValueError, 'allowlist mismatch|symlink'):
            package.verify(self.app)

    def test_non_arm64_binary_is_rejected(self):
        with patch.object(package, 'run', return_value='x86_64'):
            with self.assertRaisesRegex(ValueError, 'arm64'):
                package.inspect_binary(self.payload)

    def test_non_system_dylib_is_rejected(self):
        with patch.object(package, 'run', side_effect=['arm64', 'binary:\n\t@rpath/private.dylib (compatibility version 1)']):
            with self.assertRaisesRegex(ValueError, 'Non-system'):
                package.inspect_binary(self.payload)


class DeveloperIdentityTests(unittest.TestCase):
    DETAILS = "CodeDirectory v=20500 flags=0x10000(runtime)\nAuthority=Developer ID Application: Public Fixture (TEAM123)\nAuthority=Developer ID Certification Authority\nTeamIdentifier=TEAM123\nTimestamp=Oct 17, 2026 at 9:00:00 AM\n"

    def test_timestamped_runtime_identity_is_recorded(self):
        with patch.object(package, 'run', return_value=self.DETAILS) as native:
            details = package.developer_signature(Path('public-fixture'))
            self.assertEqual(details['team'], 'TEAM123')
            self.assertTrue(details['hardened_runtime'])
            self.assertIn('-R=anchor apple generic and certificate leaf[field.1.2.840.113635.100.6.1.13] exists', native.call_args_list[0].args)

    def test_development_certificate_is_not_a_developer_id_release_identity(self):
        with patch.object(package, 'run', return_value=self.DETAILS.replace('Developer ID Application:', 'Apple Development:')):
            with self.assertRaisesRegex(ValueError, 'Application certificate'):
                package.developer_signature(Path('public-fixture'))

    def test_timestamp_and_runtime_are_required(self):
        for details in (self.DETAILS.replace('Timestamp=', 'NoTimestamp='), self.DETAILS.replace('(runtime)', '(none)')):
            with self.subTest(details=details), patch.object(package, 'run', return_value=details):
                with self.assertRaisesRegex(ValueError, 'trusted timestamp and hardened runtime'):
                    package.developer_signature(Path('public-fixture'))


if __name__ == '__main__':
    unittest.main()
