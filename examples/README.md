# SCULPT Examples

(C) 2026 byte5 GmbH

Examples are grouped by intent so new developers can find the right starting point quickly.

All current examples use explicit provider namespaces via:
- `use(<target>.ui)`
- `use(<target>.input) as input`
Legacy shorthand (`render ...`, `key(...)`) is rejected by default.

## 1) Showcase Pairs (Low ND vs High ND)
- `showcase/games/snake_low_nd.sculpt`
- `showcase/games/snake_high_nd.sculpt`
  - Same game objective, different ND strategy.
- `showcase/gui/service_desk_low_nd.sculpt`
- `showcase/gui/service_desk_high_nd.sculpt`
  - GUI business workflow with explicit vs delegated behavior.
- `showcase/web/ops_portal_low_nd.sculpt`
- `showcase/web/ops_portal_high_nd.sculpt`
  - Web operations portal, low vs high ND.
- `showcase/cli/invoice_reconcile_low_nd.sculpt`
- `showcase/cli/invoice_reconcile_high_nd.sculpt`
  - Data-heavy CLI reconciliation, low vs high ND.

## 2) Getting Started
- `getting-started/hello_world.sculpt`
  - Smallest deterministic program.
- `getting-started/native_window.sculpt`
  - Basic GUI output with explicit target metadata.

## 3) Games
- `games/snake_portable.sculpt`
  - One portable snake script intended to run on `cli`, `gui`, and `web` with a shared gameplay core (`@meta profile=portable`).
- `games/snake_cli_low_nd_showcase.sculpt`
  - CLI snake with low ND: explicit behavior, predictable output.
- `games/snake_cli_high_nd_showcase.sculpt`
  - CLI snake with high ND: compact code, larger delegated solution space.
- `games/snake_gui_showcase.sculpt`
  - GUI-optimized snake for native desktop window behavior.
- `games/snake_web_showcase.sculpt`
  - Web-optimized snake for responsive browser delivery (`web_profile=next-app`).
- `games/breakout_high_nd.sculpt`
  - Compact high-ND Breakout that delegates mechanics/UI detail via constraints.
- `games/breakout_low_nd.sculpt`
  - Detailed low-ND Breakout with explicit flow and rule structure.
- `games/legacy/snake_high_nd_legacy.sculpt`
- `games/legacy/snake_low_nd_legacy.sculpt`
  - Archived pre-showcase versions kept for reference only.

## 4) Business
- `business/invoice_review.sculpt`
  - Simple approval/rejection flow.
- `business/incident_triage_assistant.sculpt`
  - Operational playbook assistant.
- `business/expense_approval_workflow.sculpt`
  - Realistic workflow-oriented example with strict logic and light ND.
- `business/modular_invoice_app.sculpt`
  - Multi-file business app using namespace imports (`import(Billing.Shared.InvoiceRules)`) and project file `business/modular_invoice_app.sculpt.json`.
  - Build this one via project file:
    - `sculpt build examples/business/modular_invoice_app.sculpt.json --provider stub`

## 5) Web
- `web/incident_status_dashboard.sculpt`
  - Web-target incident overview with clear state navigation (`web_profile=next-app`).
- `web/support_ticket_board.sculpt`
  - Small service-desk use case with ticket detail flow and SLA screen (`web_profile=laravel-mvc`).

## 6) Practical UI Kit (Real-App Direction)
- `practical/cli_control_center.sculpt`
  - CLI operations console with panels, queues, progress, and deployment gate flow.
- `practical/gui_service_desk.sculpt`
  - GUI service-desk workbench (list/detail/edit + confirmation flow).
- `practical/web_ops_portal.sculpt`
  - Web operations portal with dashboard, incidents, timeline, and change request action flow.

## Quick Run
```bash
sculpt build examples/getting-started/hello_world.sculpt --target cli
sculpt run examples/getting-started/hello_world.sculpt --target cli
```
