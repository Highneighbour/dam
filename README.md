# DAMM v2 Honorary Quote-Only Fee Module

This repository contains an Anchor-compatible Solana program that implements a Quote-only Honorary Fee Position with a permissionless 24h distribution crank, plus mock integration programs and a fully reproducible devcontainer + CI.

Highlights:
- Deterministic PDAs for policy, progress, treasury authority and position owner
- Quote-only enforcement with base-fee detection and hard failure
- Permissionless, paginated 24h crank with idempotency
- Mock Streamflow program for locked amount queries, and a minimal cp-amm compatibility shim for claiming fees
- Devcontainer and CI to run everything without local setup

See `INTEGRATION.md` for wiring into real programs.

## Quickstart (Devcontainer / Docker)

- Build devcontainer image and run tests:

```bash
docker build -t damm-honorary-dev .devcontainer
docker run --rm -it -v "$(pwd)":/workspace damm-honorary-dev /bin/bash -lc "./scripts/setup-local.sh && ./scripts/deploy-local.sh && ./scripts/run-tests.sh"
```

- VSCode Devcontainer:
  - Open folder in container
  - Run:

```bash
./scripts/setup-local.sh
./scripts/deploy-local.sh
anchor test
```

## Repository structure

```
damm-honorary-fee-module/
├── programs/
│   └── damm_honorary_fee/
│       ├── src/lib.rs
│       ├── src/errors.rs
│       ├── src/state.rs
│       ├── src/events.rs
│       └── tests/                # (ts tests also under /tests)
├── mock_programs/
│   ├── streamflow_mock/
│   └── cp_amm_stub/
├── deployments/local/
├── .devcontainer/
├── ci/
├── scripts/
├── README.md
└── LICENSE
```

## PDAs

- `InvestorFeePositionOwnerPda`: seeds `["vault", vault_pubkey, "investor_fee_pos_owner"]`
- `PolicyPda`: seeds `["policy", vault_pubkey]`
- `ProgressPda`: seeds `["progress", vault_pubkey]`
- `TreasuryAuthorityPda`: seeds `["treasury", vault_pubkey]`

## Errors

- NotQuoteOnly, BaseFeesObserved, DayGateNotOpen, InsufficientTreasury,
  InvalidPaginationCursor, MinPayoutNotMet, StreamflowReadError, AlreadyProcessedPage, InvalidQuoteMint

## Events

- `HonoraryPositionInitialized`
- `QuoteFeesClaimed { amount }`
- `InvestorPayoutPage { page_index, paid_total }`
- `CreatorPayoutDayClosed { day_id, remainder }`

## Day/pagination semantics

- Day boundary: `day_id = floor(unix_ts / 86400)` (UTC). First crank for a new day requires `now >= last_distribution_ts + 86400`. Subsequent pages are allowed while within the same day window.
- Pagination is idempotent via a 512-page bitmap in `ProgressPda`.

## Integration notes

- cp-amm: this repo ships a minimal `cp_amm_stub` exposing `init_pool`, `create_position`, `accrue_fees`, and `claim_fees`. For production, replace with your DAMM v2 cp-amm and adjust accounts accordingly. The `claim_fees` interface is compatible with token transfers into treasuries so base/quote deltas can be measured deterministically.
- Streamflow: `streamflow_mock` stores per-stream `locked_amount` and `y0` under PDA `["stream_lock", stream_pubkey]`. The main program reads these accounts directly. Swap the program ID to the real Streamflow program and point to their per-stream state for live integration.

## Security & invariants

- Quote-only: any non-zero base claim aborts the crank.
- All math uses integer floor division. Dust and daily cap remainder accumulate in `carry_over_lamports` and are routed to creator on day close.
- Program never performs token conversions; only quote mint is moved.
- Liveness: missing investor ATAs may be created if policy allows.

## Scripts

- `./scripts/setup-local.sh`: initialize toolchains and keys
- `./scripts/deploy-local.sh`: build programs and bring up validator
- `./scripts/run-tests.sh`: run tests

## Versions

- Rust: 1.73.0
- Anchor CLI: 0.30.1
- Solana CLI: v1.18.x

## How to run tests

```bash
./scripts/setup-local.sh
./scripts/deploy-local.sh
./scripts/run-tests.sh
```

## Swapping mocks for real integrations

- cp-amm: Replace the program ID in `Anchor.toml`, update accounts passed into instructions, and ensure `claim_fees` transfers to the provided treasuries. If the real interface differs, adapt a small shim.
- Streamflow: Replace program ID in your client and pass real per-stream state accounts. Layout must include `locked_amount` and `y0` fields.
