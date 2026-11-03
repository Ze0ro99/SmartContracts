# Local Development Environment

## Supported setup

The repository is a Cargo workspace targeting Soroban contracts. The canonical local toolchain is declared in `rust-toolchain.toml`:

- Rust 1.90.0
- `rustfmt` and `clippy`
- `wasm32-unknown-unknown`
- Python 3 (standard library only for the shared fixture validator)

The Soroban SDK version is managed by the workspace manifest and lockfile. Do not update `Cargo.lock` as part of routine environment setup.

## Option A — Dev Container

Open the repository in VS Code and select **Reopen in Container**. The devcontainer builds from the root `Dockerfile` and runs `make bootstrap`.

## Option B — Docker

```sh
docker build -t pirc-smartcontracts-dev .
docker run --rm -v "$PWD":/workspace -w /workspace pirc-smartcontracts-dev make check
docker run --rm -v "$PWD":/workspace -w /workspace pirc-smartcontracts-dev make test
```

For faster incremental builds, mount a persistent Cargo target directory:

```sh
docker run --rm -v "$PWD":/workspace -v pirc-cargo-target:/workspace/target -w /workspace pirc-smartcontracts-dev make test
```

## Option C — Native host

Install Rust using rustup, Python 3, Git, a C compiler, pkg-config, and OpenSSL development headers. The repository toolchain file selects the Rust version and WebAssembly target.

```sh
rustup show
make bootstrap
make fixture
make check
make test
make wasm
```

## Standard commands

| Command | Purpose |
|---|---|
| `make fixture` | Validate the shared JSON arithmetic fixture using Python |
| `make fmt` | Check formatting without modifying files |
| `make check` | Formatting, fixture validation, and locked workspace compilation |
| `make test` | All checks plus locked Rust workspace tests |
| `make wasm` | Build the selected contract for wasm32 |
| `make clean` | Remove local Cargo artifacts |

Set `CONTRACT` to another package name for `make wasm`, for example `make wasm CONTRACT=pirc-207-registry-v3`.

## Network and signing safety

The default workflow is local-only. Compilation and unit tests do not require a Pi or Stellar RPC endpoint, wallet, private key, seed phrase, signer, relayer, or funded account. Do not add secrets to `.env`, source files, Docker build arguments, or test fixtures.

Network deployment is intentionally outside the default Make targets. Any future Testnet operation must pin the network passphrase, RPC URL, contract allowlist, WASM hash, signer permissions, and explicit human approval. Mainnet deployment must be a separate reviewed process.

## Evidence boundary

A passing local command is local evidence only. A green GitHub Actions run must be tied to the exact commit SHA. A WASM build is not a deployment, security audit, economic backing, or proof of Pi Network integration.
