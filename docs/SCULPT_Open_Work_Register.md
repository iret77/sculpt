# SCULPT Open Work Register

(C) 2026 byte5 GmbH

This document consolidates open phases, packages, issues, todos, and product ideas that are still relevant for SCULPT. It is intentionally operational: roadmap documents describe the vision, this file tracks what still needs attention.

## Status Legend

| Status | Meaning |
|---|---|
| Open | Not started or not finished enough to rely on. |
| In Progress | Partly implemented, but not yet closed by tests, docs, and practical examples. |
| Candidate | Conceptually accepted, needs implementation planning or validation. |
| Watch | Keep visible; do not start before higher-priority work is stable. |

## 1. Open Milestones

| ID | Milestone | Status | Goal | Exit Signal |
|---|---|---|---|---|
| M1 | Foundation Stability | In Progress | Make compiler, TUI, CLI, examples, and artifacts predictable for daily use. | No blocking regressions in examples; TUI and CLI workflows behave consistently. |
| M2 | Deterministic Core | In Progress | Make contracts, strict symbols, ND constraints, replay, and validation dependable. | Invalid target symbols fail before LLM execution; replay works as a trusted CI path. |
| M3 | Team-Scale Language | Open | Make large multi-file projects workable for teams. | Namespace imports, project files, scoped rules, and diagnostics work on 100+ file projects. |
| M4 | Production Targets | In Progress | Make built-in `cli`, `gui`, and `web` targets useful beyond demos. | Practical non-demo apps pass target-specific quality gates and feel credible to users. |
| M5 | Performance And CI At Scale | Open | Keep large projects affordable and fast. | Incremental compile, compact IR defaults, cache reuse, cost/token telemetry. |
| M6 | Ecosystem And 1.0 | Open | Make providers and contracts extensible by others without losing quality. | Versioned provider SDK, compatibility checks, registry process, publish-ready spec. |

## 2. Active Priority Packages

| ID | Package | Priority | Status | Next Work |
|---|---|---|---|---|
| P1-A | Data-Path Safety Completion | P1 | In Progress | Keep hardening deterministic data workloads, artifact validation, and data benchmark diagnostics. |
| P1-B | Contract And Namespace Scalability | P1 | In Progress | Finish contract compatibility checks, symbol cataloging, namespace import diagnostics, and large-project workflows. |
| P1-C | Provider Platform Hardening | P1 | In Progress | Stabilize LLM and target provider interfaces, fallback policy, telemetry, and conformance checks. |
| P1-D | Baseline Provider Practicality | P1 | In Progress | Expand `cli`, `gui`, and `web` provider contracts until real apps can be built without undocumented magic. |
| P1-E | Example Showcase Quality | P1 | In Progress | Replace weak examples with polished, useful, target-relevant examples that demonstrate high-ND vs low-ND. |
| P1-F | Benchmark Readiness | P1 | In Progress | Do not run another official SCULPT-vs-vibe benchmark until product capability makes a win likely. |
| P2-A | Build Telemetry Expansion | P2 | In Progress | Improve run/build history, token/cost tracking, and TUI trend visibility. |
| P2-B | Dist Retention Policy | P2 | In Progress | Finish clean retention UX and document auto-clean behavior. |
| P2-C | CLI/TUI Regression Coverage | P2 | Open | Add focused tests for key TUI actions, modal flows, build/run parity, and per-script dist isolation. |
| P2-D | Prompt-Drift Competitive Benchmarking | P2 | Open | Track SCULPT vs prompt-first output drift over releases. |

## 3. Open Issues And Risks

| ID | Area | Status | Issue | Why It Matters |
|---|---|---|---|---|
| I-001 | Examples | Open | Some examples are still weaker than the vision and may look like ordinary toy demos. | First-run perception decides whether developers continue exploring SCULPT. |
| I-002 | Target Contracts | Open | Built-in contracts still expose too few useful functions for serious CLI, GUI, and Web apps. | Without practical contracts, SCULPT falls back toward vague prompting. |
| I-003 | Magic Ambiguity | In Progress | Any apparent magic words in examples must either come from a contract or from explicit `define(...)` / `?"..."` usage. | Developers need to know what is language, provider API, variable, or ND intent. |
| I-004 | GUI Target | In Progress | GUI output needs professional parity on macOS, Windows, and Linux. | Windows/Linux developers must be able to try credible GUI examples immediately. |
| I-005 | Web Target | In Progress | Web output must reflect modern app reality, not only static HTML. | Real web apps are framework-backed or frontend-app based; target providers must preserve that flexibility. |
| I-006 | TUI Polish | In Progress | The TUI still needs durable regression coverage for focus, scrolling, modals, config editing, and external interactive runs. | The TUI is part of the product experience, not only a helper. |
| I-007 | Snake Showcase | In Progress | Snake should demonstrate portable, CLI-specific, GUI-specific, and Web-specific development styles. | It is the clearest current demo for target portability versus target specialization. |
| I-008 | Benchmark Credibility | Open | A new business benchmark should only run after target/provider capability is materially stronger. | Running too early gives a noisy or predictably weak result. |
| I-009 | Context Efficiency | Open | Large programs cannot be passed wholesale through the LLM forever. | SCULPT needs incremental compile slices, dependency graphs, and compact IR reuse. |
| I-010 | Provider Ecosystem | Open | External target/LLM providers need a stable SDK, test kit, and compatibility process. | Community-maintained contracts only work if they can be validated and versioned. |

## 4. Language And Semantics Todos

| ID | Topic | Status | Todo |
|---|---|---|---|
| L-001 | Scoped Rules | In Progress | Continue supporting rules inside flows/states so source code stays local and rename-safe. |
| L-002 | Namespaces | In Progress | Harden namespace resolution and diagnostics for large domain structures such as `Billing.Account.Invoice`. |
| L-003 | Project Files | In Progress | Keep single-file scripts simple while making `.sculpt.json` project files first-class for multi-file apps. |
| L-004 | Imports | In Progress | Prefer namespace imports over file includes; avoid path-based include mechanics that fight project structure. |
| L-005 | Target Packages | In Progress | Make `use(...)` the clear bridge from SCULPT source to provider-exported functions. |
| L-006 | Soft Defines | In Progress | Document and validate reusable ND constraints defined in SCULPT, especially inside `nd(...)` blocks. |
| L-007 | Inline ND Intent | Candidate | Keep `?"..."` as explicit inline natural-language intent where structured constraints are not worth defining. |
| L-008 | Block Syntax | In Progress | Keep `:` for block openers that require `end`; keep `::` for one-line event shortcuts. |
| L-009 | Semicolon | In Progress | Treat `;` as a newline replacement for compact one-line forms, not as decorative syntax. |
| L-010 | Diagnostics | In Progress | Keep expanding semantic diagnostics (`S*`, `F*`, `R*`, `N*`, `B*`, `NS*`, `C*`, `M*`) with actionable messages. |

## 5. Contract And Provider Todos

| ID | Topic | Status | Todo |
|---|---|---|---|
| C-001 | CLI Contract | In Progress | Expand practical symbols for text UI, menus, tables, forms, file/data operations, and deterministic batch tasks. |
| C-002 | GUI Contract | In Progress | Expand window, layout, controls, validation, modal, list/table, navigation, and platform-native behavior coverage. |
| C-003 | Web Contract | In Progress | Expand app-shell, routing, forms, tables, cards, filters, persistence hooks, API calls, and adapter profiles. |
| C-004 | Data Namespace | In Progress | Keep deterministic data ops contract-validated across CLI, GUI, and Web where useful. |
| C-005 | ND Constraints | Open | Provide curated but bounded reusable constraints in contracts; avoid impossible million-entry catalogs. |
| C-006 | Contract Versioning | In Progress | Enforce compatibility between script-declared contract version and selected target provider. |
| C-007 | Contract Docs | In Progress | Keep per-target reference docs complete enough that examples contain no unexplained calls. |
| C-008 | Provider SDK | Open | Define external provider packaging, metadata, contract publication, test harness, and build/run interface. |
| C-009 | Provider Registry | Open | Design curated registry process for community providers without making compiler core depend on them. |
| C-010 | Conformance Gates | In Progress | Continue provider conformance checks for built-in and external providers. |

## 6. Target Output Todos

| ID | Target | Status | Todo |
|---|---|---|---|
| T-001 | CLI | In Progress | Keep CLI output interactive, visually credible, and useful for forms, dashboards, reports, and games. |
| T-002 | GUI | In Progress | Make generated native/desktop apps feel professional on macOS, Windows, and Linux. |
| T-003 | Web | In Progress | Support credible modern web app output, including standard static profile plus framework adapter paths. |
| T-004 | Web Stacks | In Progress | Continue `web_profile` support for `standard`, `next-app`, and `laravel-mvc`; document tradeoffs. |
| T-005 | Portable Targets | Candidate | Demonstrate one SCULPT source that runs acceptably on CLI, GUI, and Web. |
| T-006 | Specialized Targets | Candidate | Demonstrate target-specific variants that use each platform's strengths. |
| T-007 | Future Targets | Watch | Keep architecture ready for services, embedded devices, wearables, TVs, game engines, and future runtimes. |

## 7. Example And Demo Todos

| ID | Example Area | Status | Todo |
|---|---|---|---|
| E-001 | Showcase Structure | In Progress | Keep examples organized by purpose and target; avoid loose files in root example folders. |
| E-002 | High-ND vs Low-ND | In Progress | Provide visible pairs where high-ND is shorter and freer, low-ND is more controlled and predictable. |
| E-003 | Snake | In Progress | Finalize portable, CLI, GUI, and Web Snake variants; keep the CLI version visually above QBasic-era baseline. |
| E-004 | Business CLI | In Progress | Keep invoice/data examples practical and deterministic enough for benchmark relevance. |
| E-005 | Business GUI | In Progress | Keep service-desk style examples credible as small professional apps. |
| E-006 | Business Web | In Progress | Make the web portal example look like a simple but real operational theme. |
| E-007 | Single Game Limit | Open | Keep one strong game family as demo; avoid making SCULPT look like a game engine project. |
| E-008 | Comment Quality | Open | Examples should explain SCULPT concepts without drowning the code in prose. |
| E-009 | Buildability | Open | All examples must build with the current compiler and target contracts before release-facing pushes. |

## 8. TUI And CLI Todos

| ID | Area | Status | Todo |
|---|---|---|---|
| U-001 | TUI Config Editor | In Progress | Keep improving provider/model/API key editing as real form controls, not text-file editing. |
| U-002 | TUI Focus | In Progress | Make active pane visually obvious and keep scroll indicators reliable. |
| U-003 | TUI Logs | In Progress | Ensure external interactive runs restore the TUI cleanly after exit. |
| U-004 | TUI Editor Launch | Candidate | Add a clean way to open selected SCULPT files in the user's configured editor. |
| U-005 | TUI Project Files | Candidate | Show project files distinctly and offer project-aware build/run actions. |
| U-006 | CLI Progress | In Progress | Keep progress indicators correct, premium-looking, and non-noisy. |
| U-007 | CLI Help | In Progress | Keep help output aligned with branded CLI design and accurate options. |
| U-008 | Version Display | In Progress | Always display current compiler version from package metadata, not hardcoded values. |

## 9. Benchmark Todos

| ID | Benchmark | Status | Todo |
|---|---|---|---|
| B-001 | Data-Heavy | In Progress | Keep data-heavy gate green with deterministic artifacts and reproducibility checks. |
| B-002 | Workflow | In Progress | Keep workflow benchmark acceptance/reproducibility automated. |
| B-003 | UI Practical | In Progress | Keep target practical quality gates tied to meaningful UI/runtime behavior. |
| B-004 | Official Re-Run | Open | Run next serious SCULPT-vs-vibe benchmark only after readiness gates are green. |
| B-005 | Metrics | In Progress | Track acceptance rate, reproducibility, unique hashes, token usage, duration, and fix effort. |
| B-006 | Reporting | Open | Keep detailed reports separate from overview docs to avoid documentation drift. |

## 10. Documentation Todos

| ID | Doc Area | Status | Todo |
|---|---|---|---|
| D-001 | README | In Progress | Keep pitch short, current, and linked to the right deeper docs. |
| D-002 | For Dummies | In Progress | Keep explaining SCULPT concepts in plain developer language, including ND, IR, contracts, `?`, and `define(...)`. |
| D-003 | Handbook | In Progress | Keep compiler and language behavior complete but not academic. |
| D-004 | Syntax Manifest | In Progress | Keep syntax rules current, especially `:`, `::`, `;`, `end`, `use`, `import`, and `?`. |
| D-005 | Target References | In Progress | Keep CLI, GUI, and Web references synchronized with actual exported contract symbols. |
| D-006 | Roadmap | In Progress | Keep milestone document strategic, not a granular backlog. |
| D-007 | Backlog | In Progress | Decide whether this open work register should replace or complement the existing backlog. |
| D-008 | Versioning | In Progress | Keep version policy visible so future agents and contributors bump versions correctly before pushes. |

## 11. Product Ideas To Keep Visible

| ID | Idea | Status | Rationale |
|---|---|---|---|
| IDEA-001 | Contract-first IDE support | Watch | IDEs should read target contracts for completion, docs, and coloring. |
| IDEA-002 | Compact IR as LLM-native format | Open | Context-window efficiency is a core resource problem for large SCULPT apps. |
| IDEA-003 | Incremental LLM compilation | Open | Large apps need changed-unit compilation, not full-program LLM calls. |
| IDEA-004 | Freeze/replay as CI primitive | In Progress | Deterministic locked builds are essential for professional workflows. |
| IDEA-005 | Provider marketplace/registry | Watch | Community contracts/providers need discoverability and trust signals. |
| IDEA-006 | Target-specific showcase variants | In Progress | Demonstrates portability versus platform-specific strength without pretending one mode solves everything. |
| IDEA-007 | SCULPT project templates | Candidate | Project creation should offer useful starting points by intent and target. |
| IDEA-008 | Cost-aware compiler UX | Open | Token/cost/time should be visible and optimizable like CPU/RAM used to be. |
| IDEA-009 | Policy engine | Watch | Enterprises will need rules for provider use, data sensitivity, audit, and allowed targets. |
| IDEA-010 | Language server | Watch | Team-scale SCULPT requires IDE diagnostics, navigation, rename, and contract-aware completion. |

## 12. Recommended Next Execution Order

| Order | Work | Reason |
|---|---|---|
| 1 | Make all showcase examples build and run with the current compiler. | First impressions and regression safety. |
| 2 | Expand baseline target contracts for useful CLI, GUI, and Web apps. | Removes unexplained magic and enables serious examples. |
| 3 | Finish Snake multi-target demonstration. | Shows portability vs target specialization clearly. |
| 4 | Harden TUI external-run recovery and config UX. | Keeps daily workflow credible. |
| 5 | Close contract/namespace scalability gaps. | Required before large projects and team workflows. |
| 6 | Build provider SDK/conformance path. | Required before community providers are realistic. |
| 7 | Add incremental compile/cache design. | Required before large apps fit context and cost constraints. |
| 8 | Re-run business benchmark only after readiness gates pass. | Avoids measuring the wrong maturity level. |

