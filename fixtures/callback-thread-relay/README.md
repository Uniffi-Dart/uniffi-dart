# Generated callback dispatch fixture

The production generator emits an opt-in native companion and Dart wrappers.
See [the integration guide](../../docs/callback-dispatch.md) for packaging,
supported API shapes and lifetime requirements.

Run `python3 fixtures/callback-thread-relay/probe/run_generated.py --dart /path/to/dart`,
adding `--aot` for compiled execution. CI runs both modes on Linux and macOS.
The runner builds both native assets, tests the companion, and exercises generated
callbacks, owner-thread reentry, thread-local cleanup, retained handles and multiple
isolates. It includes rollback, state-mutation and companion-mismatch controls.
Each process has a bounded timeout. Reports are generated under
`target/generated-dispatch-{jit,aot}` and uploaded as CI artifacts.

The earlier handwritten prototype and its historical reports remain available at
commit `c76728a872f755cfe7d14e421d3a0ce5719822be`; they are not part of this fixture.
