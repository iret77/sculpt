# SCULPT Target References

(C) 2026 byte5 GmbH

This is the entry point for target-specific source-code references.

## Why This Exists
In SCULPT source code, statements like `on key(...)`, `render text(...)`, and ND constraints are easy to write.
What they mean at build/run time depends on:
- core language semantics, and
- the selected target contract (`cli`, `gui`, `web`, or external providers).

## Source Of Truth
- Live contract (machine-readable):
  - `sculpt target describe --target cli`
  - `sculpt target describe --target gui`
  - `sculpt target describe --target web`
- Core language semantics:
  - [SCULPT Semantics](SCULPT_Semantics.md)
- Target architecture:
  - [SCULPT Target Model](SCULPT_Target_Model.md)

## Per-Target References
- [CLI Target Reference](SCULPT_Target_CLI_Reference.md)
- [GUI Target Reference](SCULPT_Target_GUI_Reference.md)
- [Web Target Reference](SCULPT_Target_WEB_Reference.md)

## Language vs Target (Quick Rule)
- `flow`, `state`, `rule`, `nd`, `on`, `when`, `emit`, `run`, `terminate` are language-level.
- Calls like `key(...)`, `render text(...)`, and many ND call names are interpreted through target contracts and provider/runtime behavior.
