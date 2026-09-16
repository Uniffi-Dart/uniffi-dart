#!/usr/bin/env python3
"""Test generated handle bindings on Linux x64 and, optionally, ARM32 under QEMU.

The pointer mutation must pass on x64 and crash at the typed error on ARM32.
Set CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER for the cross build.
"""
import argparse
import json
import os
from pathlib import Path
import re
import resource
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--dart', required=True, type=Path)
parser.add_argument('--arm-sdk', type=Path)
parser.add_argument('--qemu', type=Path)
parser.add_argument('--sysroot', type=Path)
parser.add_argument('--gcc-lib', type=Path)
parser.add_argument('--offline', action='store_true')
parser.add_argument('--toolchain', default='1.85.1')
args = parser.parse_args()
if any([args.arm_sdk, args.qemu, args.sysroot, args.gcc_lib]) and not all([
    args.arm_sdk, args.qemu, args.sysroot, args.gcc_lib
]):
    parser.error('ARM32 requires --arm-sdk, --qemu, --sysroot and --gcc-lib')
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
fixture = Path(__file__).resolve().parents[1]
workspace = fixture.parents[1]
target = Path(os.environ.get('CARGO_TARGET_DIR', workspace / 'target/handle-validation')).resolve()
output = workspace / 'target/handle-results'
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_TARGET_DIR=str(target))
if args.offline:
    env['CARGO_NET_OFFLINE'] = 'true'
results = []

def run(name, command, cwd=workspace, crash=False, marker=None, timeout=900):
    timed_out = False
    try:
        process = subprocess.run(list(map(str, command)), cwd=cwd, env=env,
                                 stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                 text=True, timeout=timeout)
        code, log = process.returncode, process.stdout
    except subprocess.TimeoutExpired as error:
        timed_out = True
        code = None
        log = (error.stdout or b'').decode(errors='replace')
    passed = not timed_out and (code in (-6, -11, 134, 139) if crash else code == 0)
    passed = passed and (marker is None or marker in log)
    if crash:
        passed = passed and 'PASS: handles' not in log
    (output / f'{name}.log').write_text(log)
    results.append({'case': name, 'exit': code, 'expected_crash': crash,
                    'timed_out': timed_out, 'passed': passed,
                    'command': list(map(str, command))})
    (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    print(f'{name}: exit={code}, passed={passed}', flush=True)
    if not passed:
        raise RuntimeError(log[-6000:])

sdk = args.dart.resolve().parent.parent
run('build-x64', ['cargo', '+' + args.toolchain, 'build', '-p', 'uniffi-dart',
                 '-p', 'simple_iface', '--features', 'uniffi-dart/binary'])
x64lib = target / 'debug/libsimple_iface.so'
architectures = [('x64', [sdk / 'bin/dartvm'])]
assets = {'linux_x64': {'package:handle_probe/uniffi:simple_iface': ['absolute', str(x64lib)]}}
if args.arm_sdk:
    run('build-arm', ['cargo', '+' + args.toolchain, 'build', '-p', 'simple_iface',
                     '--target', 'armv7-unknown-linux-gnueabihf'])
    armlib = target / 'armv7-unknown-linux-gnueabihf/debug/libsimple_iface.so'
    assets['linux_arm'] = {'package:handle_probe/uniffi:simple_iface': ['absolute', str(armlib)]}
    architectures.append(('arm', [args.qemu, '-L', args.sysroot, '-E',
        f'LD_LIBRARY_PATH={args.sysroot}/lib:{args.gcc_lib}/lib', args.arm_sdk / 'bin/dartvm']))
config = output / 'uniffi.toml'
config.write_text('[bindings.dart]\npackage_name = "handle_probe"\ncdylib_name = "simple_iface"\n')
generated = output / 'generated'
run('generate', [target / 'debug/uniffi_bindgen_dart', '--library', x64lib,
                 '--out-dir', generated, '--config', config, '--no-format'])
source = (generated / 'simple_iface.dart').read_text()
# Deliberately regress only the failing method's ABI. Keep the Dart wrapper's
# int signature so this control tests a real native crash, not a compiler error.
symbol = 'uniffi_simple_iface_fn_method_object_fail'
pattern = r'@Native<[^@;]+external\s+void\s+' + symbol + r'\([^;]+;'
replacement = '''@Native<Void Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId, symbol: "uniffi_simple_iface_fn_method_object_fail")
external void _pointerFail(Pointer<Void> handle, Pointer<RustCallStatus> status);
void uniffi_simple_iface_fn_method_object_fail(int handle, Pointer<RustCallStatus> status) =>
  _pointerFail(Pointer<Void>.fromAddress(handle), status);
'''
mutated, count = re.subn(pattern, lambda _: replacement, source)
assert count == 1, ('expected exactly one method ABI mutation', count)
for variant, content in [('generated', source), ('high-handles', source), ('pointer-mutation', mutated)]:
    package = output / variant
    package.mkdir(exist_ok=True)
    (package / 'simple_iface.dart').write_text(content)
    runtime = (generated / 'uniffi_runtime.dart').read_text()
    if variant == 'high-handles':
        assert runtime.count('int _counter = 1;') == 1
        runtime = runtime.replace('int _counter = 1;', 'int _counter = 0xfedcba9800000001;')
    (package / 'uniffi_runtime.dart').write_text(runtime)
    (package / 'test').mkdir(exist_ok=True)
    (package / 'test/handle_cases.dart').write_text((fixture / 'test/handle_cases.dart').read_text())
    (package / 'pubspec.yaml').write_text("name: handle_probe\nenvironment:\n  sdk: '>=3.10.0 <4.0.0'\ndependencies:\n  ffi: ^2.0.1\n")
    (package / 'native_assets.yaml').write_text(json.dumps({'format-version': [1, 0, 0], 'native-assets': assets}))
    (package / 'probe.dart').write_text("""import 'dart:ffi';
import 'test/handle_cases.dart';
Future<void> main() async {
  print('ABI: ${Abi.current()}, pointer bytes: ${sizeOf<IntPtr>()}');
  await runHandleCases();
}
""")
    run(variant + '-pub', [args.dart, 'pub', 'get', *(['--offline'] if args.offline else [])], cwd=package)
    kernel = package / 'probe.dill'
    run(variant + '-kernel', [sdk / 'bin/dartaotruntime', sdk / 'bin/snapshots/gen_kernel_aot.dart.snapshot',
        '--platform', sdk / 'lib/_internal/vm_platform_strong.dill',
        '--packages', package / '.dart_tool/package_config.json',
        '--native-assets', package / 'native_assets.yaml', '--output', kernel, package / 'probe.dart'])
    for arch, prefix in architectures:
        crash = variant == 'pointer-mutation' and arch == 'arm'
        run(variant + '-' + arch, [*prefix, kernel], crash=crash, timeout=90,
            marker='before typed object error' if crash else 'PASS: handles')
abi_kernel = output / 'abi.dill'
run('recorder-kernel', [sdk / 'bin/dartaotruntime', sdk / 'bin/snapshots/gen_kernel_aot.dart.snapshot',
    '--platform', sdk / 'lib/_internal/vm_platform_strong.dill', '--output', abi_kernel,
    fixture / 'test/handle_abi_probe.dart'])
for arch, prefix in architectures:
    run('recorder-' + arch, [*prefix, abi_kernel, x64lib if arch == 'x64' else armlib], timeout=90)
print('Generated bindings and mutation controls passed. Reports:', output)
