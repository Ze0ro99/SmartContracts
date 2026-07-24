#### `scripts/generate_diagrams.py`

```python
#!/usr/bin/env python3
"""
Generate professional engineering diagrams (Mermaid) for any smart contract.
"""

from pathlib import Path
import argparse

def generate_diagrams(contract_name: str, output_dir: str = "reports"):
    diagrams_dir = Path(output_dir) / contract_name / "diagrams"
    diagrams_dir.mkdir(parents=True, exist_ok=True)

    sequence = """sequenceDiagram
    participant Merchant
    participant Contract
    participant Token as Stellar Token
    participant Subscriber

    Merchant->>Contract: register_service()
    Subscriber->>Contract: subscribe() + approve()
    Contract->>Token: transfer / transfer_from
    loop Billing Cycle
        Merchant->>Contract: process()
        Contract->>Token: transfer_from (batch)
    end
    Subscriber->>Contract: cancel() / toggle_auto_renew()
"""

    state = """stateDiagram-v2
    [*] --> ServiceRegistered
    ServiceRegistered --> ActiveSubscription: subscribe
    ActiveSubscription --> Charging: process
    Charging --> ActiveSubscription: success
    Charging --> AutoRenewOff: failure
    ActiveSubscription --> Cancelled: cancel
    Cancelled --> [*]
"""

    component = """graph TD
    Merchant --> Contract
    Subscriber --> Contract
    Contract --> TokenContract
    Contract --> EventsStorage
    PiRCStandards --> Contract
    Admin --> Contract
"""

    (diagrams_dir / "sequence.mmd").write_text(sequence, encoding="utf-8")
    (diagrams_dir / "state.mmd").write_text(state, encoding="utf-8")
    (diagrams_dir / "component.mmd").write_text(component, encoding="utf-8")

    print(f"Diagrams generated → {diagrams_dir}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate engineering diagrams")
    parser.add_argument("--contract", required=True, help="Contract directory name")
    parser.add_argument("--output", default="reports", help="Output directory")
    args = parser.parse_args()
    generate_diagrams(args.contract, args.output)
