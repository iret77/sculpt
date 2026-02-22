# SCULPT Versioning

(C) 2026 byte5 GmbH

## Scope

SCULPT versioning has two independent layers:

1. **Language version** (for `.sculpt` source compatibility)
2. **Component version** (compiler, providers, targets, tooling)

---

## 1) Language Version Policy

Format: `MAJOR.MINOR` (example: `1.0`)

- `MINOR` updates are **backward-compatible** within the same major.
- `MAJOR` updates may contain breaking syntax/semantic changes.

Current baseline:
- **default language:** `1.0`
- **supported range:** `>=1.0 <2.0`

### Source Pinning

Projects should pin the intended language version in source metadata:

```sculpt
@meta language=1.0
```

This keeps builds stable when newer compiler versions are installed.

---

## 2) Component Version Policy

Format: SemVer `MAJOR.MINOR.PATCH`

Applies to:
- `sculpt` compiler binary
- target providers
- LLM providers
- ecosystem tooling

Rules:
- Components can release independently.
- Bugfix/security releases do **not** require a language version change.
- Components must explicitly declare which language range they support.

### Mandatory Rule Before Push

If any of these paths changed, bump `Cargo.toml` package version **before push**:
- `src/**`
- `tests/**`
- `examples/**`
- `ir-schemas/**`
- `Cargo.lock`

This rule is enforced by:
- local script: `scripts/ensure_version_bump.sh`
- CI workflow: `.github/workflows/version-guard.yml`

### Bump Strategy

- `PATCH` (`x.y.z -> x.y.z+1`): bugfixes, UX improvements, non-breaking behavior changes.
- `MINOR` (`x.y.z -> x.y+1.0`): notable feature additions that stay backward-compatible.
- `MAJOR` (`x.y.z -> x+1.0.0`): breaking CLI/compiler behavior or compatibility changes.

Language version (`1.0`) is independent and changes only with language policy decisions.

---

## 3) User-Facing Transparency Rules

Compiler output should expose language support in key places:
- `sculpt --version`
- `sculpt help`
- build/freeze/replay/run headers
- `sculpt target list`

This ensures developers can always see:
- current compiler version
- default SCULPT language version
- supported language range

---

## 4) Compatibility Contract (Required)

Each component should declare:

- `supportsLanguage`: version range (example: `>=1.0 <2.0`)
- `defaultLanguage`: default compile language (example: `1.0`)
- `componentVersion`: SemVer of the component itself

This allows safe mixed-version toolchains and predictable upgrades.

---

## 5) Release Checklist (Short)

1. Update `Cargo.toml` version.
2. Update `CHANGELOG.md`.
3. Run `cargo test`.
4. Run `scripts/ensure_version_bump.sh <base-ref> HEAD`.
5. Push only after all checks pass.
