# Data-Heavy Dataset Spec: Invoice Reconciliation

(C) 2026 byte5 GmbH

## Purpose

Define canonical datasets for the SCULPT-vs-vibe data-heavy benchmark so both approaches are tested on identical, repeatable inputs.

## Files

- `invoices.csv`
- `payments.csv`

## Common CSV Rules

- UTF-8, comma-separated, header row required
- Decimal separator: `.`
- Date format: `YYYY-MM-DD`
- Currency: `EUR`
- IDs are case-sensitive

## Schema

### invoices.csv

| Column | Type | Required | Notes |
|---|---|---|---|
| `invoice_id` | string | yes | unique in normal data, duplicates intentionally injected in edge cases |
| `customer_id` | string | yes | customer key |
| `amount_due` | number(2) | yes | expected amount |
| `currency` | string | yes | fixed `EUR` for this benchmark |
| `invoice_date` | date | yes | issue date |
| `due_date` | date | yes | due date |
| `status_hint` | string | no | optional source hint, not authoritative |

### payments.csv

| Column | Type | Required | Notes |
|---|---|---|---|
| `payment_id` | string | yes | unique payment id |
| `invoice_id` | string | no | may be empty or wrong in edge cases |
| `customer_id` | string | yes | customer key |
| `amount_paid` | number(2) | yes | paid amount |
| `currency` | string | yes | fixed `EUR` |
| `payment_date` | date | yes | payment date |
| `reference` | string | no | free-form reference |

## Matching Rules (Benchmark Ground Truth)

1. Primary match: same `invoice_id` and same `customer_id`.
2. Amount tolerance: exact (`0.00`) for baseline tests.
3. Date tolerance: payment can be `-7..+30` days around `due_date`.
4. If invoice id missing on payment:
   - fallback candidate by `customer_id` + unique exact `amount_due == amount_paid` inside date tolerance.
5. If multiple candidates exist, classify as `ambiguous` (do not auto-resolve).

## Required Classifications

- `matched_full`
- `matched_partial`
- `overpaid`
- `missing_payment`
- `duplicate_payment`
- `ambiguous`
- `suspicious`

## Dataset Sizes

### Small

- invoices: `1,000`
- payments: `1,200`
- used for quick iteration and correctness smoke test

### Medium

- invoices: `20,000`
- payments: `25,000`
- used for baseline benchmark run

### Large

- invoices: `100,000`
- payments: `130,000`
- used for stress/repro run

## Edge Case Injection Targets

Inject the following ratios into medium/large datasets:

- exact full matches: `~70%`
- partial payments: `~8%`
- overpayments: `~4%`
- missing payments: `~10%`
- duplicate payments: `~5%`
- ambiguous candidates: `~2%`
- suspicious references (mismatched customer/id anomalies): `~1%`

## Benchmark Output Contract

The implementation under test must generate:

- `reconciliation_report.json`
- `exceptions.csv`
- terminal summary line with counts per classification

`reconciliation_report.json` must include at least:

- `input_stats`
- `classification_counts`
- `processing_ms`
- `rules_version`
- `generated_at`

## Determinism Requirement

For identical inputs and same implementation revision:

- `reconciliation_report.json` hash must be stable
- `exceptions.csv` row order must be stable (sort by `invoice_id`, then `payment_id`)
