#!/usr/bin/env python3
"""Generate a source inventory report; this script does not certify security or deployment."""
from __future__ import annotations
import argparse
import datetime as dt
import json
import re
from pathlib import Path

def load_pirc_assets(imported_path="imported/pirc"):
    assets = {"schemas": 0, "registries": 0, "manifests": 0}
    base = Path(imported_path)
    if base.exists():
        for path in base.rglob("*.json"):
            name = path.name.lower()
            if "registry" in name:
                assets["registries"] += 1
            elif "manifest" in name:
                assets["manifests"] += 1
            else:
                assets["schemas"] += 1
    return assets

def generate_report(contract_name, output_dir="reports", deep=False):
    if not re.fullmatch(r"[A-Za-z0-9_-]+", contract_name):
        raise ValueError("Contract name contains unsupported characters")
    contract_dir = Path("contracts") / contract_name
    if not (contract_dir / "Cargo.toml").is_file():
        raise FileNotFoundError("Contract manifest not found: " + str(contract_dir / "Cargo.toml"))
    source_dir = contract_dir / "src"
    source_files = sorted(source_dir.rglob("*.rs")) if source_dir.exists() else []
    public_functions = set()
    lines_of_code = 0
    for source in source_files:
        source_text = source.read_text(encoding="utf-8", errors="replace")
        lines_of_code += len(source_text.splitlines())
        public_functions.update(re.findall(r"\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)", source_text))
    generated_at = dt.datetime.now(dt.timezone.utc).isoformat()
    report = {
        "contract": contract_name,
        "generated_at": generated_at,
        "mode": "deep" if deep else "standard",
        "source_files": len(source_files),
        "lines_of_code": lines_of_code,
        "public_functions": sorted(public_functions),
        "pirc_imported_assets": load_pirc_assets(),
        "build_status": "NOT_REPORTED_BY_THIS_SCRIPT",
        "unit_test_status": "NOT_REPORTED_BY_THIS_SCRIPT",
        "security_review_status": "NOT_PERFORMED_BY_THIS_SCRIPT",
        "testnet_status": "NOT_DEPLOYED_BY_THIS_SCRIPT",
        "limitations": [
            "Source inventory is not a security audit.",
            "Compilation and tests must be evidenced by their CI jobs.",
            "Deployment requires separate manual approval and network read-back."
        ]
    }
    report_dir = Path(output_dir) / contract_name
    report_dir.mkdir(parents=True, exist_ok=True)
    (report_dir / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    markdown = [
        "# Source Inventory Report — " + contract_name,
        "",
        "- Generated: " + generated_at,
        "- Mode: " + report["mode"],
        "- Rust source files: " + str(len(source_files)),
        "- Lines of code: " + str(lines_of_code),
        "- Public functions discovered: " + str(len(public_functions)),
        "- Build/test/security/deployment status: Not asserted by this generator",
        "",
        "## Evidence boundaries",
        "",
        "- This document inventories source files only.",
        "- Build and test results must be read from the corresponding CI jobs.",
        "- This report does not establish audit completion, Testnet deployment, or Mainnet readiness.",
        "",
        "## Public function names",
        ""
    ]
    markdown.extend("- " + name for name in sorted(public_functions))
    (report_dir / "REPORT.md").write_text("\n".join(markdown) + "\n", encoding="utf-8")
    return report_dir

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", required=True)
    parser.add_argument("--output-dir", default="reports")
    parser.add_argument("--deep", action="store_true")
    args = parser.parse_args()
    print("Report written to " + str(generate_report(args.contract, args.output_dir, args.deep)))

if __name__ == "__main__":
    main()
