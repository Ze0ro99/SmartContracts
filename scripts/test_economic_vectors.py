#!/usr/bin/env python3
"""Validate canonical economic settlement vectors using only the Python standard library.

This validates fixture arithmetic, not Soroban execution or chain state.
"""
from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "economic-engine-vectors.json"


def main() -> int:
    data = json.loads(FIXTURE.read_text(encoding="utf-8"))
    cases = {case["id"]: case for case in data["cases"]}

    ledger = cases["net-distributable"]["input"]
    deductions = sum(
        ledger[key]
        for key in (
            "operating_costs",
            "tax_and_statutory",
            "refunds_and_due_liabilities",
            "reserve_replenishment",
            "approved_commitments",
        )
    )
    assert ledger["collected_revenue"] - deductions == cases["net-distributable"]["expected"]["distributable"]

    allocation = cases["largest-remainder-equal-weights"]
    amount = allocation["input"]["amount"]
    participants = sorted(allocation["input"]["participants"], key=lambda item: item["id"])
    total_weight = sum(item["weight"] for item in participants)
    shares = []
    for item in participants:
        quotient, remainder = divmod(amount * item["weight"], total_weight)
        shares.append({"id": item["id"], "amount": quotient, "remainder": remainder})
    remaining = amount - sum(item["amount"] for item in shares)
    for item in sorted(shares, key=lambda item: (-item["remainder"], item["id"]))[:remaining]:
        item["amount"] += 1
    actual = [{"id": item["id"], "amount": item["amount"]} for item in shares]
    assert actual == allocation["expected"]["allocations"]
    assert sum(item["amount"] for item in actual) == amount

    reserve = cases["reserve-coverage"]
    assert reserve["input"]["reserve"] * 10_000 // reserve["input"]["liabilities"] == reserve["expected"]["coverage_bps"]

    growth = cases["supply-growth"]
    assert growth["input"]["net_issuance"] * 10_000 // growth["input"]["previous_supply"] == growth["expected"]["growth_bps"]

    underfunded = cases["underfunded-period"]
    gross = underfunded["input"]["collected_revenue"]
    total = sum(value for key, value in underfunded["input"].items() if key != "collected_revenue")
    assert total > gross
    assert underfunded["expected"]["error"] == "UNDERFUNDED_PERIOD"

    print(f"Validated {len(cases)} economic settlement vectors.")
    print("Scope: fixture arithmetic only; not Soroban contract parity.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
