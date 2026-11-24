# specifications/

Only the command-name map lives here (`command-names/`): the command, tool
and argument names Operandi takes so scripts port (docs/COMMAND_NAMES.md).

The behaviour specs and their test data were moved to a private repository
on 2026-11-23. The tests that read them build only when that checkout is
found next to this one (`../Operandi-specs`, or `OPERANDI_SPECS`); see
`crates/operandi-core/build_specs.rs`.
