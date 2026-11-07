# Solana Transfer Extraction (Substreams)

A Substreams module (Rust) that extracts transfers from Solana blocks **385870151 to 385870156** and outputs one JSON object per transfer:

```json
{"from": "<sender>", "to": "<recipient>", "amount": 325253}
```

## What it extracts

The `map_transfers` module processes each Solana block and emits:

- **Native SOL transfers**: System Program `Transfer` instruction
- **SPL token transfers**: Token Program and Token-2022 `Transfer` and `TransferChecked` instructions

Inner (CPI) instructions are included. For SPL transfers, `from` and `to` are the token accounts used in the instruction.

A Substreams map module emits one message per block, so a small `jq` step flattens each block's list into one JSON object per transfer.

## Project structure

```
proto/transfers.proto            output message definitions
src/lib.rs                       map_transfers module
substreams.yaml                  manifest
solana-transfers-v0.1.0.spkg     packaged module
output.jsonl                     final output, one transfer per line
raw.jsonl                        raw per-block Substreams output
```

## Prerequisites

- Rust and the `wasm32-unknown-unknown` target
- Substreams CLI
- `buf` CLI (used by `substreams protogen`)
- `jq`
- A free Pinax API key, set as `SUBSTREAMS_API_KEY`

## Build

```bash
rustup target add wasm32-unknown-unknown
substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"
cargo build --target wasm32-unknown-unknown --release
substreams pack substreams.yaml
```

## Run

The stop block is exclusive, so 385870157 includes block 385870156.

```bash
export SUBSTREAMS_API_KEY="<your key>"

substreams run -e solana.substreams.pinax.network:443 \
  solana-transfers-v0.1.0.spkg map_transfers \
  -s 385870151 -t 385870157 -o jsonl > raw.jsonl

jq -c '.["@data"].transfers[]? | {from, to, amount: ((.amount // "0") | tonumber)}' raw.jsonl > output.jsonl
```

Notes on the `jq` step:
- Protobuf JSON prints `uint64` as a string, so `tonumber` converts `amount` to an integer.
- Zero amounts are omitted from Substreams JSON output, so `// "0"` fills them in.

## Result

`output.jsonl` contains 3974 transfers for blocks 385870151-385870156, one JSON object per line.

## Note on the sample transfer

The sample in the brief (`Eq1c3uNu...` to `4XpSNfo...`, amount 325253) does not appear in my output. To check this, I ran a temporary debug module (not committed) that scans every instruction for either address or the 325253 amount, over blocks 385870151-385870299. No instruction contained `4XpSNfo...` or that amount. `Eq1c3uNu...` only appeared as a receiver (8500000 lamports) in that range. I believe the sample is illustrative, but I'd like to confirm which instruction types or block range are expected.

## Process notes

- Tools used: Claude (explanations, code, debugging), Substreams and Pinax docs
- Environment: GitHub Codespaces (Linux)
- Problems hit and how I solved them:
  - Provider auth: the old token-exchange URL returned `null`; switched to a Pinax free API key
  - `buf` CLI missing for `protogen`: installed it
  - Build errors: added the missing `prost-types` dependency and borrowed the account values in `bs58::encode`
  - `jq` failed on zero amounts: defaulted missing `amount` to 0
  - Sample transfer missing from output: added SPL token support, then debugged with the byte-level search described above
