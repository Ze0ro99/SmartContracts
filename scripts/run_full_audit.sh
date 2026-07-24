#!/usr/bin/env bash
set -euo pipefail

CONTRACT="${1:-subscription}"

echo "Starting professional audit for contract: $CONTRACT"

python3 scripts/generate_professional_report.py --contract "$CONTRACT"
python3 scripts/generate_diagrams.py --contract "$CONTRACT"

if [ -f "contracts/$CONTRACT/Cargo.toml" ]; then
  echo "Running cargo tests..."
  (cd "contracts/$CONTRACT" && cargo test -- --nocapture) || true
fi

echo "Audit completed successfully for $CONTRACT"
echo "Reports available at: reports/$CONTRACT/"
