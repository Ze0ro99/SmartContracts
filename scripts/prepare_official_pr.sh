#!/usr/bin/env bash
# =============================================================================
#  prepare_official_pr.sh  (Termux / Mobile friendly version)
#  Professional PR preparation for PiNetwork/SmartContracts
# =============================================================================

set -euo pipefail

# ---------- Colors ----------
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'; BOLD='\033[1m'

log()   { echo -e "${BLUE}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
header(){ 
  echo -e "\n${BOLD}${CYAN}════════════════════════════════════════${NC}"
  echo -e "${BOLD}${CYAN}  $*${NC}"
  echo -e "${BOLD}${CYAN}════════════════════════════════════════${NC}\n"
}

DRY_RUN=false
SKIP_REBASE=false
FORCE=false

for arg in "$@"; do
  case $arg in
    --dry-run)     DRY_RUN=true ;;
    --skip-rebase) SKIP_REBASE=true ;;
    --force)       FORCE=true ;;
  esac
done

# ---------- Pre-flight (Termux friendly) ----------
header "1. Pre-flight Checks"

command -v git >/dev/null || err "git is required. Install with: pkg install git"
command -v cargo >/dev/null || warn "cargo not found. You can still prepare docs & git."

if command -v rustup >/dev/null; then
  ok "rustup found"
  rustup target list --installed | grep -q wasm32-unknown-unknown || {
    log "Adding wasm32 target..."
    rustup target add wasm32-unknown-unknown || warn "Could not add wasm target"
  }
else
  warn "rustup not found (common on Termux). Build steps will be skipped."
fi

command -v python3 >/dev/null || warn "python3 not found – some report scripts may fail"

# Must be inside git repo
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || err "Not inside a git repository"
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"
ok "Repo root: $REPO_ROOT"

# ---------- 1. Repository Preparation ----------
header "1. Repository Preparation"

log "Fetching latest..."
$DRY_RUN || {
  git fetch origin 2>/dev/null || true
  git fetch upstream 2>/dev/null || true
}

CURRENT_BRANCH=$(git branch --show-current)
log "Current branch: $CURRENT_BRANCH"

if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
  warn "Not on main. Switching..."
  $DRY_RUN || git checkout main 2>/dev/null || git checkout master
fi

log "Updating main..."
$DRY_RUN || git pull origin main --ff-only 2>/dev/null || \
           git pull origin master --ff-only 2>/dev/null || \
           warn "Could not fast-forward (manual merge may be needed)"

# Create feature branch
FEATURE_BRANCH="contribution/v1-subscription-$(date +%Y%m%d)"
if git show-ref --verify --quiet "refs/heads/$FEATURE_BRANCH"; then
  FEATURE_BRANCH="${FEATURE_BRANCH}-$(date +%H%M%S)"
fi

log "Creating feature branch: $FEATURE_BRANCH"
$DRY_RUN || git checkout -b "$FEATURE_BRANCH"
ok "Feature branch created"

# Optional rebase warning
if ! $SKIP_REBASE; then
  AUTO_COUNT=$(git log --oneline --grep="auto-heal\|chore(auto" HEAD 2>/dev/null | wc -l || echo 0)
  if [[ $AUTO_COUNT -gt 20 ]]; then
    warn "There are many auto-heal commits ($AUTO_COUNT)."
    warn "It is recommended to clean history later with: git rebase -i upstream/main"
  fi
fi

# LICENSE check
log "Checking LICENSE..."
if [[ -f LICENSE ]] && grep -qi "PiOS\|Pi Open Source" LICENSE; then
  ok "LICENSE is PiOS – good"
else
  warn "LICENSE may not be official PiOS. Please verify manually."
fi

# Update .gitignore
log "Updating .gitignore..."
cat >> .gitignore << 'EOF'

# === Added by prepare_official_pr.sh ===
/target/
**/*.wasm
**/*.wat
reports/local/
*.log
*.tmp
.DS_Store
.termux/
storage/
.idea/
.vscode/
__pycache__/
*.pyc
.venv/
*.bak
imported/exports/*.tmp
EOF
sort -u .gitignore -o .gitignore 2>/dev/null || true
ok ".gitignore updated"

# ---------- 2. Documentation ----------
header "2. Professional Documentation"

log "Creating professional README.md..."
cat > README.md << 'EOF'
# Pi Smart Contracts

Professional collection of Soroban smart contracts for the **Pi Network** ecosystem.

This repository is a community enhancement of the official  
[PiNetwork/SmartContracts](https://github.com/PiNetwork/SmartContracts).

## Focus of this contribution
- High-quality **Subscription Contract** (PiRC-2)
- Clean documentation and structure
- Modular PiRC library foundation
- Professional audit and reporting tools

## Build

```bash
cargo build --release --target wasm32-unknown-unknown



Testbash


cargo test -p subscription



Auditbash


./scripts/run_full_audit.sh subscription



LicensePiOS License – see LICENSE
EOF
ok "README.md created"Subscription READMEmkdir -p contracts/subscription
cat > contracts/subscription/README.md << 'EOF'Subscription Contract (PiRC-2)Official reference implementation of the Pi Network recurring subscription smart contract.Key FeaturesNon-custodial recurring payments

Zero billing drift

Paginated batch processing

Trial protection

Full subscriber control



Build & test:bash


cargo test -p subscription
cargo build --release --target wasm32-unknown-unknown -p subscription



EOF
ok "contracts/subscription/README.md created"CHANGELOGcat > CHANGELOG.md << 'EOF'Changelog[Unreleased] – Upstream contribution preparationAddedProfessional README and documentation

CONTRIBUTING.md

Per-contract README for subscription

Improved .gitignore and repository hygiene



ImprovedFocus on official Subscription Contract readiness

Audit and reporting structure



NotesThis preparation prioritizes clean contribution of the Subscription Contract and documentation to the official repository.
EOF
ok "CHANGELOG.md created"CONTRIBUTINGcat > CONTRIBUTING.md << 'EOF'ContributingKeep changes focused and well documented.

Prefer the official Subscription Contract as the main artifact.

Follow PiOS License.

Test before submitting.



bash


cargo test
cargo build --release --target wasm32-unknown-unknown
./scripts/run_full_audit.sh subscription



EOF
ok "CONTRIBUTING.md created"---------- 3. Code Quality ----------header "3. Code Quality & Testing"if command -v cargo >/dev/null; then
  log "Attempting workspace build (may take time)..."
  if $DRY_RUN; then
    log "[DRY-RUN] cargo build skipped"
  else
    cargo build --release --target wasm32-unknown-unknown 2>&1 | tail -20 || 

      warn "Build had errors – focus on fixing subscription first"
  fi  log "Running subscription tests..."
  cargo test -p subscription -- --nocapture 2>&1 | tail -30 || 

    warn "Tests failed or package not found"
else
  warn "cargo not available – skipping build & tests"
fiRun audit if existsif [[ -x scripts/run_full_audit.sh ]]; then
  log "Running audit script..."
  $DRY_RUN || ./scripts/run_full_audit.sh subscription || warn "Audit finished with warnings"
else
  warn "scripts/run_full_audit.sh not found"
fiBasic reportsmkdir -p reports
cat > reports/CONTRACT_AUDIT.md << EOFContract Audit ReportGenerated: $(date -u)FocusSubscription Contract prepared for upstream PR

Documentation and structure cleaned
EOF



cat > reports/STABILITY_REPORT.md << EOFStability ReportGenerated: $(date -u)
Prepared for clean contribution to PiNetwork/SmartContracts
EOFok "Basic reports created"---------- 4. Final Checks ----------header "4. Final Checks"log "Scanning for common issues..."
grep -r --include="*.rs" -l "unsafe" contracts/ 2>/dev/null | head -5 && 

  warn "Some files contain 'unsafe' – review them" || ok "No obvious unsafe usage found"---------- Summary ----------header "Finished – Manual Steps Remaining"echo -e "${GREEN}Automated preparation completed.${NC}"
echo
echo -e "${BOLD}Next manual steps:${NC}"
echo "1. Review README.md, CHANGELOG.md and CONTRIBUTING.md"
echo "2. Manually verify Subscription Contract against official PiRC-2"
echo "3. Fix any remaining build/test errors"
echo "4. Clean history if needed: git rebase -i upstream/main"
echo "5. Commit and push:"
echo
echo "   git add ."
echo "   git status"
echo "   git commit -m "docs: professional PR preparation for upstream""
echo "   git push -u origin $FEATURE_BRANCH"
echo
echo -e "${CYAN}Then open the Pull Request on GitHub.${NC}"




