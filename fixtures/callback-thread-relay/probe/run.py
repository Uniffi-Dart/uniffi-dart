#!/usr/bin/env python3
"""Generated-UniFFI relay experiment with crash and mutation controls."""
import argparse
import hashlib
import json
import os
import platform
from pathlib import Path
import re
import shutil
import signal
import struct
import subprocess
import sys
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--dart', default=os.environ.get('DART', 'dart'))
parser.add_argument('--toolchain', default='1.85.1')
parser.add_argument('--offline', action='store_true', help='Use cached Cargo and Dart dependencies only')
parser.add_argument('--timeout', type=int, default=120, help='Watchdog seconds per Dart runtime case')
parser.add_argument('--cases', default='direct,thread,parallel,clone,errors,nested,repeat,cross-nested,retained,retained-clone-repeat,shutdown-pending')
parser.add_argument('--aot', action='store_true', help='Compile and run native executables instead of JIT')
args = parser.parse_args()
if args.timeout <= 0:
    parser.error('--timeout must be positive')
if struct.calcsize('P') != 8:
    parser.error('This prototype requires a 64-bit host; the 32-bit handle ABI is not covered')
if sys.platform not in ('linux', 'darwin', 'win32'):
    parser.error('This runner currently targets Linux, macOS and Windows desktop hosts')
if os.name != 'nt':
    import resource
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
fixture = Path(__file__).resolve().parents[1]
workspace = fixture.parents[1]
target = Path(os.environ.get('CARGO_TARGET_DIR', workspace / 'target' / 'relay-validation')).resolve()
output = workspace / 'target' / ('relay-results-aot' if args.aot else 'relay-results')
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_TARGET_DIR=str(target))
if args.offline:
    env['CARGO_NET_OFFLINE'] = 'true'
results = []
def git_output(*argv):
    result = subprocess.run(['git', *argv], cwd=workspace, capture_output=True,
                            text=True, encoding='utf-8', errors='replace')
    return result.stdout.strip() if result.returncode == 0 else None

report = {
    'source_commit': git_output('rev-parse', 'HEAD'),
    'source_changes': git_output('status', '--porcelain'),
    'platform': platform.platform(),
    'architecture': platform.machine(),
    'python': platform.python_version(),
    'offline_requested': args.offline,
    'timeout_seconds': args.timeout,
    'execution_mode': 'aot' if args.aot else 'jit',
    'cases': results,
}
def save_report():
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

def command(label, argv, cwd=workspace, timeout=120, expected='success', marker=None):
    process_options = ({'creationflags': subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == 'nt'
                       else {'start_new_session': True})
    started = time.monotonic()
    process = subprocess.Popen(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, text=True, encoding='utf-8',
                               errors='replace', **process_options)
    timed_out = False
    try:
        stdout, _ = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        if os.name == 'nt':
            subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if process.poll() is None:
                process.kill()
        else:
            os.killpg(process.pid, signal.SIGKILL)
        stdout, _ = process.communicate()
        stdout += f'\nWATCHDOG TIMEOUT after {timeout}s\n'
    (output / f'{label}.log').write_text(stdout)
    code = process.returncode
    if expected == 'crash':
        ok = code != 0 and 'Cannot invoke native callback outside an isolate' in stdout
    elif expected == 'state-failure':
        ok = code != 0 and 'owner state was not updated' in stdout
    else:
        ok = code == 0
    ok = ok and not timed_out and (marker is None or marker in stdout)
    results.append({'case': label, 'exit': code, 'expectation': expected, 'passed': ok,
                    'timed_out': timed_out, 'seconds': round(time.monotonic() - started, 2)})
    save_report()
    print(f'{label}: exit={code}, expected={expected}, passed={ok}', flush=True)
    if not ok:
        print(stdout[-6000:])
        raise RuntimeError(f'{label} failed its control expectation')
    return stdout

report['dart'] = command('dart-version', [args.dart, '--version']).strip()
report['rust'] = command('rust-version', ['rustc', '+' + args.toolchain, '--version']).strip()
command('build', ['cargo', '+' + args.toolchain, 'build', '-p', 'uniffi-dart',
                 '-p', 'callback_thread_relay', '--features', 'uniffi-dart/binary'], timeout=900)
library_name = {'linux': 'libcallback_thread_relay.so', 'darwin': 'libcallback_thread_relay.dylib',
                'win32': 'callback_thread_relay.dll'}[sys.platform]
shared = target / 'debug' / library_name
config = output / 'uniffi.toml'
config.write_text('[bindings.dart]\npackage_name = "relay_probe"\ncdylib_name = "callback_thread_relay"\n')
generated = output / 'generated'
generated.mkdir(exist_ok=True)
generator_name = 'uniffi_bindgen_dart.exe' if os.name == 'nt' else 'uniffi_bindgen_dart'
command('generate', [str(target / 'debug' / generator_name), '--library', str(shared),
                    '--out-dir', str(generated), '--config', str(config), '--no-format'])
component = generated / 'callback_thread_relay.dart'
original = component.read_text()
report['generated_sha256'] = hashlib.sha256(original.encode()).hexdigest()

# Native symbol annotations alone enable the relay. The generated method bodies,
# argument/result converters, callback methods, clone/free code remain unchanged.
replacements = {
 'uniffi_callback_thread_relay_fn_init_callback_vtable_sink': 'relay_init_callback_vtable_sink',
 **{f'uniffi_callback_thread_relay_fn_func_{name}': f'relay_{name}' for name in [
     'call_bytes_direct', 'call_bytes_thread', 'call_bytes_parallel',
     'call_checked_thread', 'call_clone_thread']},
 **{f'uniffi_callback_thread_relay_fn_func_{name}': f'relay_{name}' for name in [
     'call_cross_nested', 'call_saved_thread', 'save_sink']},
}
report['symbol_overrides'] = replacements
def relay_symbols(source):
    for name, symbol in replacements.items():
        matches = list(re.finditer(r'\bexternal\s+\S+\s+' + re.escape(name) + r'\s*\(', source))
        if len(matches) != 1:
            raise RuntimeError(f'Expected one native declaration for {name}, got {len(matches)}')
        start = source.rfind('@Native<', 0, matches[0].start())
        asset = source.index('assetId:', start, matches[0].start())
        source = source[:asset] + f"symbol: '{symbol}',\n  " + source[asset:]
    return source

for variant in ['baseline', 'relay', 'mutation']:
    package = output / variant
    if package.exists(): shutil.rmtree(package)
    shutil.copytree(generated, package)
    shutil.copy2(shared, package / shared.name)
    shutil.copy2(fixture / 'probe' / 'probe.dart', package / 'probe.dart')
    source = original if variant == 'baseline' else relay_symbols(original)
    # Read-only instrumentation to prove both native AND Dart registries drain.
    source += '\nint relayProbeHandleCount() => FfiConverterCallbackInterfaceSink._handleMap.relayProbeSize;\n'
    (package / component.name).write_text(source)
    runtime_matches = 0
    for path in package.rglob('*.dart'):
        content = path.read_text()
        marker = 'class UniffiHandleMap<T> {'
        if marker in content:
            runtime_matches += 1
            path.write_text(content.replace(marker, marker + '\nint get relayProbeSize => _map.length;', 1))
    if runtime_matches != 1: raise RuntimeError('Expected one generated handle map')
    if variant == 'mutation':
        # Same relay and result, but omit the Dart state update. If this passes,
        # the test would be accepting a stubbed result instead of actual state.
        path = package / 'probe.dart'
        content = path.read_text()
        if content.count('return ++value;') != 1:
            raise RuntimeError('Expected one callback state mutation site')
        path.write_text(content.replace('return ++value;', 'return value + 1;'))
    (package / 'pubspec.yaml').write_text('''name: relay_probe
version: 0.0.0
environment:
  sdk: '>=3.10.0 <4.0.0'
dependencies:
  ffi: ^2.0.1
  code_assets: any
  hooks: any
''')
    (package / 'hook').mkdir(exist_ok=True)
    (package / 'hook' / 'build.dart').write_text('''import 'package:hooks/hooks.dart';
import 'package:code_assets/code_assets.dart';
void main(List<String> args) async {
  await build(args, (input, output) async {
    output.assets.code.add(CodeAsset(
      package: input.packageName,
      name: 'uniffi:callback_thread_relay',
      linkMode: DynamicLoadingBundled(),
      file: input.packageRoot.resolve('__NATIVE_LIBRARY__'),
    ));
  });
}
'''.replace('__NATIVE_LIBRARY__', library_name))
    command(f'{variant}-pub', [args.dart, 'pub', 'get', *(['--offline'] if args.offline else [])], cwd=package)
    if args.aot:
        bundle = package / 'aot'
        executable = bundle / 'bundle' / 'bin' / ('probe.exe' if os.name == 'nt' else 'probe')
        command(f'{variant}-compile', [args.dart, 'build', 'cli', '--target', 'probe.dart', '-o', str(bundle)], cwd=package)
        runtime = [str(executable)]
    else:
        runtime = [args.dart, 'run', 'probe.dart']
    if variant == 'baseline':
        command('baseline-direct', [*runtime, 'baseline-direct'], cwd=package,
                timeout=args.timeout, marker='PASS baseline-direct:')
        command('baseline-thread', [*runtime, 'baseline-thread'], cwd=package,
                timeout=args.timeout, expected='crash')
    elif variant == 'mutation':
        command('mutation-state', [*runtime, 'thread'], cwd=package,
                timeout=args.timeout, expected='state-failure')
    else:
        for case in args.cases.split(','):
            command('relay-' + case, [*runtime, case], cwd=package,
                    timeout=args.timeout, marker=f'PASS {case}:')

save_report()
print('All controls and relay cases passed. Logs:', output)
