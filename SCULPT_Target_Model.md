# SCULPT Target Model (Foundation Draft)

(C) 2026 byte5 GmbH

## Goal
Provide a future-proof target architecture that scales beyond current platforms:
- desktop apps,
- web applications,
- server workloads,
- game engines/consoles,
- wearables, TVs, and IoT devices,
- frameworks that do not exist yet.

## 1) Core Principle
Do not encode platform/framework details into language-level target names.

SCULPT separates:
- **Intent** (what you build),
- **Runtime** (where it runs),
- **Provider** (how it is implemented and built).

## 2) Conceptual Layers

### 2.1 Intent Layer (`surface`)
High-level software shape:
- `cli`
- `app`
- `web`
- `service`
- `game`
- `embedded`

### 2.2 Runtime Layer (`runtime`)
Execution environment:
- `desktop`
- `mobile`
- `browser`
- `server`
- `console`
- `wearable`
- `tv`
- `iot`

### 2.3 Platform Layer (`platform` / `os` / `device`)
Optional concrete constraints:
- OS: `windows`, `macos`, `linux`, `ios`, `android`, `tvos`, `watchos`, ...
- Device classes: `xbox`, `playstation`, `nintendo`, custom embedded targets, etc.

### 2.4 Provider Layer
Concrete implementation and build runtime:
- native toolkits/frameworks,
- web frameworks,
- game engines,
- enterprise runtimes.

Examples:
- desktop app via native Swift
- desktop app via .NET
- game via Unity provider
- web via Node/Next provider
- backend via Laravel provider

## 3) Capability-First Contracts
Providers declare capability contracts instead of hardcoded language assumptions.

Examples:
- `ui.window`
- `ui.controls.button`
- `input.keyboard`
- `graphics.3d`
- `audio.spatial`
- `network.http`
- `storage.secure`
- `notifications.push`

The compiler validates requested capabilities before LLM compile and before deterministic build.

## 4) Resolver Model
Given script metadata + CLI flags, the resolver:
1. identifies requested intent/runtime/platform constraints,
2. finds providers whose contracts satisfy required capabilities,
3. picks best match (priority/order rules),
4. fails with a clear diagnostic if no compatible provider exists.

## 5) Backward Compatibility (Current State)
Current CLI uses:
- `--target cli|gui|web`

Mapping for transition:
- `cli` -> `surface=cli`, `runtime=desktop|server`
- `gui` -> `surface=app`, `runtime=desktop`
- `web` -> `surface=web`, `runtime=browser`

This mapping is transitional and keeps current scripts working.

## 6) Why This Scales
- New targets do not require language redesign.
- New frameworks and devices become providers.
- Teams can standardize contracts across mixed ecosystems.
- Future platforms can be integrated with minimal compiler-core changes.

## 7) Implementation Phases

### Phase A
- Keep current `--target` flags.
- Introduce internal target intent/runtime model.
- Add provider capability metadata and resolver.

### Phase B
- Add CLI flags:
  - `--surface`
  - `--runtime`
  - `--platform` (optional)
- Keep `--target` as compatibility alias.

### Phase C
- Contract-first provider registry.
- Priority strategy for provider selection.
- Multi-provider build profiles.

### Phase D
- Full ecosystem mode:
  - game engine providers,
  - enterprise runtime providers,
  - wearable/TV/console providers.

