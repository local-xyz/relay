# Grid integration

This repository starts at Ithaca relay commit `bbbaf3f7ac50a3cf886cf9544733b317f03fdca9` (v26.1.4). The complete upstream commit history is retained. It is a separate checkout beside Grid, not a Grid submodule.

## Changes from upstream

- Decode `KeyType.External` and `SpendPeriod.Forever`, plus the errors and refund events in Grid's current account contracts. External keys can be returned by `wallet_getKeys`; creating quotes with an external signer still returns `UnsupportedKeyType`, as upstream does. This does not add external signer simulation.
- Test against `local-xyz/account-contracts` at `b7629795541242d0c7fe7a3d0a315ec09873c6f1`, the revision Grid pins. Preserve v0.4 compatibility.
- Bind to the account contract as `GridAccount` (`src/types/account.rs`). That revision renamed `IthacaAccount` to `GridAccount` and set its EIP-712 domain to `GridAccount` `0.5.11`; the chain diagnostics in `src/diagnostics/chain.rs` require the delegation implementation to report that domain name, so this relay does not pass diagnostics against implementations deployed as `IthacaAccount`.
- Restore the configurable email sender and CLI/config-only environment handling from Grid commit `029d3501`. Resend keys are excluded from saved configuration.
- Restore verified-email evidence from Grid commit `d673e498`: pending emails must not count as verified. Both memory and PostgreSQL storage are covered.
- Enable PostgreSQL TLS from Grid commit `1e7c8bda` for encrypted database connections.
- Update only the upstream lockfile packages with security fixes (TLS, AWS-LC, bytes, h2, rand, ruint, time, and crossbeam), retaining compatible dependency ranges. Two maintenance-only advisories remain documented in `deny.toml`; no vulnerability advisory is suppressed.
- Use LocalProtocol's identity domain by default. Every image is published to `ghcr.io/local-xyz/relay` (linux/amd64 and linux/arm64): releases as `<version>` and `latest`, candidates as the short commit.

The upstream transaction, quote, gas estimation, and settlement algorithms are retained.

## Domains and email

| Service | Production | Staging |
| --- | --- | --- |
| Relay | `https://relay.localprotocol.xyz` | `https://relay-staging.localprotocol.xyz` |
| Identity and email verification | `https://id.localprotocol.xyz` | `https://id-staging.localprotocol.xyz` |
| Sender | `LocalProtocol <no-reply@mail.localprotocol.xyz>` | Same verified sender |

These match the existing Grid, Porto, and infrastructure configuration. Relay email configuration:

```sh
export RESEND_API_KEY='<injected secret>'
export EMAIL_SENDER='LocalProtocol <no-reply@mail.localprotocol.xyz>'
export PORTO_BASE_URL='id.localprotocol.xyz'
export ONRAMP_WORKER_SECRET='<injected secret>'
```

Use `PORTO_BASE_URL=id-staging.localprotocol.xyz` for staging. The value is a host without `https://`; the relay creates `/email/verify` links. Email delivery requires an explicit sender. Keep credentials in environment secrets, never in committed files.

The relay verifies email ownership and stores contact information. Porto still performs account authentication; this patch does not replace passkeys with email-only authentication.

## Validation and publishing

```sh
cargo test --locked --lib -- --skip test_comprehensive_erc20_mapping_discovery --skip test_layerzero_diagnostics_ethereum_mainnet
# With a disposable PostgreSQL database:
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres SQLX_OFFLINE=true cargo test --locked --lib postgres_email_attestation -- --ignored
# With Foundry and cargo-nextest installed, and private submodules initialized:
cargo e2e -- --locked --test tests -E 'test(e2e::cases::simple::) | test(e2e::cases::keys::)'
```

The two skipped unit tests query an external Ithaca mainnet RPC. Local account tests deploy the pinned account contracts to isolated Anvil instances.

The `docker` workflow publishes to `ghcr.io/local-xyz/relay`. A manual run (`workflow_dispatch`) publishes only the short commit, so a candidate can be tested with Grid without moving `latest` or any version tag. Pushing a `v*` tag publishes `<tag>`, `latest`, and the short commit. The package must grant this repository Actions access with write permission. Image source and revision labels identify the standalone relay repository and commit.

Grid can select a candidate without changing its defaults:

```sh
VITE_RELAY_VERSION=<short-commit> VITE_DEFAULT_ENV=anvil pnpm test --run packages/account/src/viem/RelayActions.test.ts
```

If the package has not granted this repository write access, the publish step fails. Grant it under the package's settings (Manage Actions access, `local-xyz/relay`, write), or publish from a machine with a GHCR login:

```sh
docker buildx build --platform linux/amd64,linux/arm64 --build-arg BUILD_PROFILE=release \
  -t ghcr.io/local-xyz/relay:v<version> -t ghcr.io/local-xyz/relay:latest --push .
```

This keeps publishing credentials out of repository secrets.
