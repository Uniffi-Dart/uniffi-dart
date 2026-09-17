#!/usr/bin/env python3
"""Exercise opt-in generated callback dispatch and its native companion."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--dart', default='dart')
parser.add_argument('--toolchain', default='1.85.1')
parser.add_argument('--offline', action='store_true')
parser.add_argument('--aot', action='store_true')
args = parser.parse_args()
fixture = Path(__file__).resolve().parents[1]
workspace = fixture.parents[1]
target = Path(os.environ.get('CARGO_TARGET_DIR', workspace / 'target/relay-validation')).resolve()
output = workspace / 'target' / ('generated-dispatch-aot' if args.aot else 'generated-dispatch-jit')
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_TARGET_DIR=str(target))
if args.offline: env['CARGO_NET_OFFLINE'] = 'true'
if os.name != 'nt':
    import resource
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
def git(*argv):
    result = subprocess.run(['git', *argv], cwd=workspace, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else None
report = {'platform': platform.platform(), 'mode': 'aot' if args.aot else 'jit',
          'commit': git('rev-parse', 'HEAD'), 'changes': git('status', '--porcelain'), 'cases': []}
def run(name, argv, cwd=workspace, timeout=900, failure=None, marker=None):
    options = {'start_new_session': True} if os.name != 'nt' else {}
    start = time.monotonic()
    process = subprocess.Popen(list(map(str,argv)), cwd=cwd, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, text=True, **options)
    timed_out = False
    try: log, _ = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        if os.name == 'nt':
            subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
        else: os.killpg(process.pid, signal.SIGKILL)
        log, _ = process.communicate()
    (output / f'{name}.log').write_text(log)
    passed = not timed_out and ((process.returncode != 0 and failure in log) if failure else process.returncode == 0)
    if marker: passed = passed and marker in log
    report['cases'].append({'name':name, 'exit':process.returncode, 'passed':passed,
                            'timed_out':timed_out, 'seconds':round(time.monotonic()-start,2)})
    (output / 'results.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'{name}: exit={process.returncode}, passed={passed}', flush=True)
    if not passed: raise RuntimeError(log[-8000:])

suffix = '.dll' if os.name == 'nt' else ('.dylib' if platform.system() == 'Darwin' else '.so')
prefix = '' if os.name == 'nt' else 'lib'
library = prefix + 'callback_thread_relay' + suffix
companion = prefix + 'callback_thread_relay_dart_dispatch' + suffix
generator = target / 'debug' / ('uniffi_bindgen_dart.exe' if os.name == 'nt' else 'uniffi_bindgen_dart')
run('build', ['cargo','+'+args.toolchain,'build','-p','uniffi-dart','-p','callback_thread_relay','--features','uniffi-dart/binary'])
config = output / 'uniffi.toml'
config.write_text('[bindings.dart]\npackage_name = "relay_probe"\ncdylib_name = "callback_thread_relay"\ncallback_dispatch = true\n')
package = output / 'package'
run('generate', [generator, '--library', target/'debug'/library, '--out-dir', package, '--config', config, '--no-format'])
run('companion-build', ['cargo','+'+args.toolchain,'build','--manifest-path', package/'uniffi_dispatch/Cargo.toml'])
run('companion-tests', ['cargo','+'+args.toolchain,'test','--manifest-path', package/'uniffi_dispatch/Cargo.toml'])
run('companion-clippy', ['cargo','+'+args.toolchain,'clippy','--manifest-path', package/'uniffi_dispatch/Cargo.toml','--all-targets','--','-D','warnings'])
shutil.copyfile(target/'debug'/library, package/library)
shutil.copyfile(target/'debug'/companion, package/companion)
shutil.copyfile(fixture/'probe/generated.dart', package/'probe.dart')
(package/'pubspec.yaml').write_text("name: relay_probe\nenvironment:\n  sdk: '>=3.10.0 <4.0.0'\ndependencies:\n  ffi: ^2.0.1\n  hooks: any\n  code_assets: any\n")
(package/'hook').mkdir(exist_ok=True)
(package/'hook/build.dart').write_text('''import 'package:hooks/hooks.dart';
import 'package:code_assets/code_assets.dart';
void main(List<String> args) async {
  await build(args, (input, output) async {
    for (final entry in <String, String>{
      'uniffi:callback_thread_relay': '@LIBRARY@',
      'uniffi:callback_thread_relay_dispatch': '@COMPANION@',
    }.entries) {
      output.assets.code.add(CodeAsset(package: input.packageName, name: entry.key,
        linkMode: DynamicLoadingBundled(), file: input.packageRoot.resolve(entry.value)));
    }
  });
}
'''.replace('@LIBRARY@',library).replace('@COMPANION@',companion))
run('pub', [args.dart,'pub','get',*(['--offline'] if args.offline else [])],cwd=package)
run('format', [args.dart,'format','callback_thread_relay.dart','uniffi_runtime.dart','probe.dart'],cwd=package)
if args.aot:
    run('aot-build', [args.dart,'build','cli','--target','probe.dart','-o','aot'],cwd=package)
    runtime = [package/'aot/bundle/bin'/('probe.exe' if os.name=='nt' else 'probe')]
else: runtime = [args.dart,'run','probe.dart']
for case in ['direct','thread','parallel','clone','errors','nested','cross-nested','retained','retained-clone','repeat','isolates','rollback']:
    run(case,[*runtime,case],cwd=package,timeout=45,marker='PASS generated '+case)
# A stubbed return must not satisfy the original-object state assertion.
probe = package/'probe.dart'
original = probe.read_text()
assert original.count('return ++value;') == 1
probe.write_text(original.replace('return ++value;','return value + 1;'))
try:
    if args.aot:
        run('mutation-build',[args.dart,'build','cli','--target','probe.dart','-o','mutation-aot'],cwd=package)
        mutation = [package/'mutation-aot/bundle/bin'/('probe.exe' if os.name=='nt' else 'probe')]
    else: mutation = runtime
    run('state-control', [*mutation,'thread'],cwd=package,timeout=45,failure='thread callback')
finally: probe.write_text(original)
component = package/'callback_thread_relay.dart'
original_component = component.read_text()
marker = '_uniffiDispatchUnregister(endpoint, native);'
assert original_component.count(marker) == 1
component.write_text(original_component.replace(marker, ''))
try:
    if args.aot:
        run('rollback-mutation-build',[args.dart,'build','cli','--target','probe.dart','-o','rollback-mutation-aot'],cwd=package)
        mutation = [package/'rollback-mutation-aot/bundle/bin'/('probe.exe' if os.name=='nt' else 'probe')]
    else: mutation = runtime
    run('rollback-control', [*mutation,'rollback'],cwd=package,timeout=45,failure='failed lowering leaked its native route')
finally: component.write_text(original_component)
print('Generated dispatch cases and failure controls passed. Reports:', output)
