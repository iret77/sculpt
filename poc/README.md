# POC Index

(C) 2026 byte5 GmbH

This folder is intentionally curated to keep case-study evidence readable.

## Primary Documents
- [SCULPT_Case_Studies_Overview.md](SCULPT_Case_Studies_Overview.md)
- [POC_Incident_Triage_Report.md](POC_Incident_Triage_Report.md)
- [SCULPT_vs_Vibe_Incident_Triage.md](SCULPT_vs_Vibe_Incident_Triage.md)
- [SCULPT_vs_Vibe_Data_Heavy_Setup.md](SCULPT_vs_Vibe_Data_Heavy_Setup.md)
- [SCULPT_vs_Vibe_Data_Heavy_Report.md](SCULPT_vs_Vibe_Data_Heavy_Report.md)
- [SCULPT_vs_Vibe_Data_Heavy_Dataset_Spec.md](SCULPT_vs_Vibe_Data_Heavy_Dataset_Spec.md)
- [SCULPT_vs_Vibe_Data_Heavy_Execution_Checklist.md](SCULPT_vs_Vibe_Data_Heavy_Execution_Checklist.md)

## Canonical Artifacts (Kept Uncompressed)
- `classic_incident_triage.ts` (classical baseline source)
- `vibe_incident_triage.ts` (final vibe-generated source)
- `vibe_prompts_incident_triage.md` (prompt + iteration history)
- `vibe_metrics.json`
- `vibe_repro_metrics.json`
- `gates/incident_triage_vibe_gate.json`
- `data_heavy_vibe_metrics.template.json`
- `data_heavy_vibe_repro_metrics.template.json`
- `gates/data_heavy_vibe_gate.template.json`
- `data_heavy_vibe_metrics.json`
- `data_heavy_vibe_repro_metrics.json`
- `data_heavy_sculpt_metrics.json`
- `data_heavy_sculpt_repro_metrics.json`
- `data_heavy_vibe_prompts.md`
- `gates/data_heavy_vibe_gate.json`
- `data_heavy_openai_attempt.log`
- `gates/data_heavy_vibe_gate_result.txt`

## Archived Raw Iteration Artifacts
- `artifacts/incident_triage_raw_runs.zip`
  - contains `vibe_run/*` and `vibe_repro_runs/*` raw files

## Data-Heavy Utilities
- `generate_data_heavy_benchmark_data.py` (dataset generator)
- release gate automation:
  - `../scripts/ci_benchmark_release_gate.sh`
  - evaluates fresh SCULPT benchmark + SCULPT internal gate + competitive SCULPT-vs-vibe gate
- generated datasets:
  - `data/small/invoices.csv`
  - `data/small/payments.csv`
  - `data/medium/invoices.csv`
  - `data/medium/payments.csv`
  - `data/large/invoices.csv`
  - `data/large/payments.csv`
