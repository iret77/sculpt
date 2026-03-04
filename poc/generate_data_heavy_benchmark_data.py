#!/usr/bin/env python3
"""
Generate canonical benchmark CSV datasets for invoice reconciliation.

Usage:
  python3 poc/generate_data_heavy_benchmark_data.py --size small --out poc/data/small
  python3 poc/generate_data_heavy_benchmark_data.py --size medium --out poc/data/medium
  python3 poc/generate_data_heavy_benchmark_data.py --size large --out poc/data/large
"""

from __future__ import annotations

import argparse
import csv
import os
import random
from dataclasses import dataclass
from datetime import date, timedelta
from pathlib import Path


SIZE_CONFIG = {
    "small": (1000, 1200),
    "medium": (20000, 25000),
    "large": (100000, 130000),
}


@dataclass
class Invoice:
    invoice_id: str
    customer_id: str
    amount_due: float
    invoice_date: date
    due_date: date


def money(v: float) -> str:
    return f"{v:.2f}"


def write_csv(path: Path, header: list[str], rows: list[list[str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(header)
        w.writerows(rows)


def make_data(size: str, out_dir: Path, seed: int) -> None:
    inv_count, pay_target = SIZE_CONFIG[size]
    rng = random.Random(seed)

    start = date(2025, 1, 1)

    invoices: list[Invoice] = []
    inv_rows: list[list[str]] = []

    for i in range(inv_count):
        inv_id = f"INV-{i+1:07d}"
        cust = f"CUST-{rng.randint(1, max(100, inv_count // 20)):05d}"
        amount = round(rng.uniform(50.0, 5000.0), 2)
        inv_date = start + timedelta(days=rng.randint(0, 365))
        due_date = inv_date + timedelta(days=rng.randint(7, 45))
        invoices.append(Invoice(inv_id, cust, amount, inv_date, due_date))
        inv_rows.append(
            [
                inv_id,
                cust,
                money(amount),
                "EUR",
                inv_date.isoformat(),
                due_date.isoformat(),
                "",
            ]
        )

    pay_rows: list[list[str]] = []
    payment_id = 1

    for inv in invoices:
        r = rng.random()

        if r < 0.70:
            amt = inv.amount_due
            pdate = inv.due_date + timedelta(days=rng.randint(-5, 20))
            pay_rows.append(
                [
                    f"PAY-{payment_id:08d}",
                    inv.invoice_id,
                    inv.customer_id,
                    money(amt),
                    "EUR",
                    pdate.isoformat(),
                    "full-match",
                ]
            )
            payment_id += 1

        elif r < 0.78:
            amt = round(inv.amount_due * rng.uniform(0.2, 0.9), 2)
            pdate = inv.due_date + timedelta(days=rng.randint(-5, 20))
            pay_rows.append(
                [
                    f"PAY-{payment_id:08d}",
                    inv.invoice_id,
                    inv.customer_id,
                    money(amt),
                    "EUR",
                    pdate.isoformat(),
                    "partial",
                ]
            )
            payment_id += 1

        elif r < 0.82:
            amt = round(inv.amount_due * rng.uniform(1.05, 1.4), 2)
            pdate = inv.due_date + timedelta(days=rng.randint(-5, 20))
            pay_rows.append(
                [
                    f"PAY-{payment_id:08d}",
                    inv.invoice_id,
                    inv.customer_id,
                    money(amt),
                    "EUR",
                    pdate.isoformat(),
                    "overpay",
                ]
            )
            payment_id += 1

        elif r < 0.87:
            amt = inv.amount_due
            pdate = inv.due_date + timedelta(days=rng.randint(-5, 20))
            for _ in range(2):
                pay_rows.append(
                    [
                        f"PAY-{payment_id:08d}",
                        inv.invoice_id,
                        inv.customer_id,
                        money(amt),
                        "EUR",
                        pdate.isoformat(),
                        "duplicate",
                    ]
                )
                payment_id += 1

        elif r < 0.89:
            amt = inv.amount_due
            wrong_cust = f"CUST-{rng.randint(1, max(100, inv_count // 20)):05d}"
            pdate = inv.due_date + timedelta(days=rng.randint(-5, 20))
            pay_rows.append(
                [
                    f"PAY-{payment_id:08d}",
                    inv.invoice_id,
                    wrong_cust,
                    money(amt),
                    "EUR",
                    pdate.isoformat(),
                    "suspicious-customer-mismatch",
                ]
            )
            payment_id += 1

        elif r < 0.91:
            amt = inv.amount_due
            pdate = inv.due_date + timedelta(days=rng.randint(-5, 20))
            pay_rows.append(
                [
                    f"PAY-{payment_id:08d}",
                    "",
                    inv.customer_id,
                    money(amt),
                    "EUR",
                    pdate.isoformat(),
                    "missing-invoice-id",
                ]
            )
            payment_id += 1

        else:
            pass  # missing payment

    while len(pay_rows) < pay_target:
        fake_inv = f"INV-X-{rng.randint(1, inv_count*2):07d}"
        cust = f"CUST-{rng.randint(1, max(100, inv_count // 20)):05d}"
        amt = round(rng.uniform(20.0, 7000.0), 2)
        pdate = start + timedelta(days=rng.randint(0, 420))
        pay_rows.append(
            [
                f"PAY-{payment_id:08d}",
                fake_inv,
                cust,
                money(amt),
                "EUR",
                pdate.isoformat(),
                "noise",
            ]
        )
        payment_id += 1

    rng.shuffle(pay_rows)

    write_csv(
        out_dir / "invoices.csv",
        [
            "invoice_id",
            "customer_id",
            "amount_due",
            "currency",
            "invoice_date",
            "due_date",
            "status_hint",
        ],
        inv_rows,
    )

    write_csv(
        out_dir / "payments.csv",
        [
            "payment_id",
            "invoice_id",
            "customer_id",
            "amount_paid",
            "currency",
            "payment_date",
            "reference",
        ],
        pay_rows,
    )

    print(f"Generated {size} dataset in {out_dir}")
    print(f"  invoices: {len(inv_rows)}")
    print(f"  payments: {len(pay_rows)}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--size", choices=["small", "medium", "large"], required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    out = Path(args.out)
    os.makedirs(out, exist_ok=True)
    make_data(args.size, out, args.seed)
