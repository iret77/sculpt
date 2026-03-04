#!/usr/bin/env node
/* Prompt-first style baseline: single-file reconciliation CLI */

const fs = require('fs');
const path = require('path');

function parseArgs(argv) {
  const out = {
    invoices: '',
    payments: '',
    outDir: '.',
    toleranceDays: 30,
    amountTolerance: 0,
  };
  for (let i = 2; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--invoices') out.invoices = argv[++i] || '';
    else if (a === '--payments') out.payments = argv[++i] || '';
    else if (a === '--out-dir') out.outDir = argv[++i] || '.';
    else if (a === '--tolerance-days') out.toleranceDays = Number(argv[++i] || '30');
    else if (a === '--amount-tolerance') out.amountTolerance = Number(argv[++i] || '0');
  }
  if (!out.invoices || !out.payments) {
    console.error('Usage: node vibe_invoice_reconciliation.js --invoices <file> --payments <file> [--out-dir <dir>] [--tolerance-days 30] [--amount-tolerance 0]');
    process.exit(2);
  }
  return out;
}

function readCsv(file) {
  const txt = fs.readFileSync(file, 'utf8');
  const lines = txt.split(/\r?\n/).filter(Boolean);
  if (lines.length === 0) return [];
  const headers = lines[0].split(',').map((s) => s.trim());
  const rows = [];
  for (let i = 1; i < lines.length; i += 1) {
    const cols = lines[i].split(',');
    const rec = {};
    for (let j = 0; j < headers.length; j += 1) rec[headers[j]] = (cols[j] || '').trim();
    rows.push(rec);
  }
  return rows;
}

function daysDiff(a, b) {
  const da = new Date(a + 'T00:00:00Z');
  const db = new Date(b + 'T00:00:00Z');
  return Math.round((da.getTime() - db.getTime()) / 86400000);
}

function approxEq(a, b, tol) {
  return Math.abs(a - b) <= tol;
}

function hashString(s) {
  const crypto = require('crypto');
  return crypto.createHash('sha256').update(s).digest('hex');
}

function reconcile(invoices, payments, toleranceDays, amountTolerance) {
  const byInv = new Map();
  const byCust = new Map();
  for (const p of payments) {
    const inv = p.invoice_id || '';
    const cust = p.customer_id || '';
    if (!byInv.has(inv)) byInv.set(inv, []);
    byInv.get(inv).push(p);
    if (!byCust.has(cust)) byCust.set(cust, []);
    byCust.get(cust).push(p);
  }

  const usedPayment = new Set();
  const exceptions = [];
  const counts = {
    matched_full: 0,
    matched_partial: 0,
    overpaid: 0,
    missing_payment: 0,
    duplicate_payment: 0,
    ambiguous: 0,
    suspicious: 0,
  };

  for (const inv of invoices) {
    const amountDue = Number(inv.amount_due || 0);
    const dueDate = inv.due_date || '';

    const direct = (byInv.get(inv.invoice_id) || []).filter((p) => p.customer_id === inv.customer_id);
    const inWindow = direct.filter((p) => Math.abs(daysDiff(p.payment_date, dueDate)) <= toleranceDays);

    let candidates = inWindow;

    if (candidates.length === 0) {
      const custPool = byCust.get(inv.customer_id) || [];
      const fallback = custPool.filter((p) => {
        const amount = Number(p.amount_paid || 0);
        return (!p.invoice_id || p.invoice_id === '') && approxEq(amount, amountDue, amountTolerance) && Math.abs(daysDiff(p.payment_date, dueDate)) <= toleranceDays;
      });
      if (fallback.length === 1) candidates = fallback;
      if (fallback.length > 1) {
        counts.ambiguous += 1;
        exceptions.push([inv.invoice_id, '', 'ambiguous', 'multiple fallback candidates']);
        continue;
      }
    }

    if (candidates.length === 0) {
      counts.missing_payment += 1;
      exceptions.push([inv.invoice_id, '', 'missing_payment', 'no payment found']);
      continue;
    }

    if (candidates.length > 1) {
      counts.duplicate_payment += 1;
      exceptions.push([inv.invoice_id, candidates.map((p) => p.payment_id).join('|'), 'duplicate_payment', 'multiple payments for one invoice']);
      for (const p of candidates) usedPayment.add(p.payment_id);
      continue;
    }

    const p = candidates[0];
    usedPayment.add(p.payment_id);

    const amount = Number(p.amount_paid || 0);
    if (p.customer_id !== inv.customer_id) {
      counts.suspicious += 1;
      exceptions.push([inv.invoice_id, p.payment_id, 'suspicious', 'customer mismatch']);
      continue;
    }

    if (approxEq(amount, amountDue, amountTolerance)) {
      counts.matched_full += 1;
    } else if (amount < amountDue) {
      counts.matched_partial += 1;
      exceptions.push([inv.invoice_id, p.payment_id, 'matched_partial', `paid ${amount.toFixed(2)} / due ${amountDue.toFixed(2)}`]);
    } else {
      counts.overpaid += 1;
      exceptions.push([inv.invoice_id, p.payment_id, 'overpaid', `paid ${amount.toFixed(2)} / due ${amountDue.toFixed(2)}`]);
    }
  }

  for (const p of payments) {
    if (usedPayment.has(p.payment_id)) continue;
    if (!p.invoice_id) {
      counts.suspicious += 1;
      exceptions.push(['', p.payment_id, 'suspicious', 'unmatched payment without invoice_id']);
      continue;
    }
    const exists = invoices.some((i) => i.invoice_id === p.invoice_id);
    if (!exists) {
      counts.suspicious += 1;
      exceptions.push([p.invoice_id, p.payment_id, 'suspicious', 'references unknown invoice']);
    }
  }

  exceptions.sort((a, b) => {
    const ia = a[0] || '';
    const ib = b[0] || '';
    if (ia !== ib) return ia < ib ? -1 : 1;
    const pa = a[1] || '';
    const pb = b[1] || '';
    return pa < pb ? -1 : pa > pb ? 1 : 0;
  });

  return { counts, exceptions };
}

function main() {
  const args = parseArgs(process.argv);
  const t0 = Date.now();

  const invoices = readCsv(args.invoices);
  const payments = readCsv(args.payments);
  const result = reconcile(invoices, payments, args.toleranceDays, args.amountTolerance);

  fs.mkdirSync(args.outDir, { recursive: true });
  const report = {
    input_stats: {
      invoices: invoices.length,
      payments: payments.length,
    },
    classification_counts: result.counts,
    processing_ms: Date.now() - t0,
    rules_version: '1.0',
    generated_at: new Date().toISOString(),
  };

  const reportPath = path.join(args.outDir, 'reconciliation_report.json');
  const exceptionsPath = path.join(args.outDir, 'exceptions.csv');

  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n', 'utf8');
  const exLines = ['invoice_id,payment_id,classification,reason'];
  for (const row of result.exceptions) exLines.push(row.map((x) => String(x).replace(/,/g, ';')).join(','));
  fs.writeFileSync(exceptionsPath, exLines.join('\n') + '\n', 'utf8');

  const summary = Object.entries(result.counts)
    .map(([k, v]) => `${k}=${v}`)
    .join(' ');
  console.log(`summary ${summary}`);
  console.log(`report ${reportPath}`);
  console.log(`exceptions ${exceptionsPath}`);
  console.log(`hash ${hashString(fs.readFileSync(reportPath, 'utf8') + '\n' + fs.readFileSync(exceptionsPath, 'utf8'))}`);
}

main();
