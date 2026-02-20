# SCULPT Examples

(C) 2026 byte5 GmbH

Examples are grouped by intent so new developers can find the right starting point quickly.

## 1) Getting Started
- `getting-started/hello_world.sculpt`
  - Smallest deterministic program.
- `getting-started/native_window.sculpt`
  - Basic GUI output with explicit target metadata.

## 2) Games
- `games/snake_high_nd.sculpt`
  - Very high ND, minimal source code.
- `games/snake_low_nd.sculpt`
  - Low ND, explicit game rules and data.
- `games/breakout_cli.sculpt`
  - CLI arcade example with rule-driven physics and constrained ND level layout.

## 3) Business
- `business/invoice_review.sculpt`
  - Simple approval/rejection flow.
- `business/incident_triage_assistant.sculpt`
  - Operational playbook assistant.
- `business/expense_approval_workflow.sculpt`
  - Realistic workflow-oriented example with strict logic and light ND.

## 4) Web
- `web/incident_status_dashboard.sculpt`
  - Web-target incident overview with clear state navigation.
- `web/support_ticket_board.sculpt`
  - Small service-desk use case with ticket detail flow and SLA screen.

## Quick Run
```bash
sculpt build examples/getting-started/hello_world.sculpt --target cli
sculpt run examples/getting-started/hello_world.sculpt --target cli
```
