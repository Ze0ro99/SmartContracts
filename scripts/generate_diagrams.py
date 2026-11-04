#!/usr/bin/env python3
"""Generate a Mermaid source map for one Soroban contract."""
from __future__ import annotations

import argparse
import re
from pathlib import Path


def generate_diagrams(contract_name: str, output_dir: str = "reports") -> Path:
    if not re.fullmatch(r"[A-Za-z0-9_-]+", contract_name):
        raise ValueError("Contract name contains unsupported characters")
    source_dir = Path("contracts") / contract_name / "src"
    if not source_dir.is_dir():
        raise FileNotFoundError("Contract source directory not found: " + str(source_dir))

    functions = set()
    for path in sorted(source_dir.rglob("*.rs")):
        source = path.read_text(encoding="utf-8", errors="replace")
        functions.update(re.findall(r"\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)", source))

    output_dir_path = Path(output_dir) / contract_name
    output_dir_path.mkdir(parents=True, exist_ok=True)
    lines = [
        "flowchart TD",
        '  caller["Caller / dApp"] --> contract["' + contract_name + ' contract"]',
    ]
    for index, function in enumerate(sorted(functions)[:40]):
        lines.append('  contract --> fn' + str(index) + '["' + function + '()"]')
    lines.append("  contract --> storage[(Contract Storage)]")
    target = output_dir_path / "ARCHITECTURE.mmd"
    target.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return target


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", required=True)
    parser.add_argument("--output", default="reports")
    args = parser.parse_args()
    print("Diagram written to " + str(generate_diagrams(args.contract, args.output)))


if __name__ == "__main__":
    main()
