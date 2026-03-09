#!/bin/bash

# Not gonna waste a day trying to make this look fancy with ai and then rewriting it

set -e

echo "=============================================="
echo "Setting up Environment and Running InheritNext"
echo "=============================================="

echo "Starting dfx locally"

if dfx ping; then
    echo "-> DFX running Stopping it and reRunning"
    dfx stop
else
    echo "-> DFX not running starting it"
fi

dfx start --clean --background

echo "Choosing Default Identity for minter chage below if you need to change"
dfx identity use default
MINTER=$(dfx identity get-principal)
echo "Minter: -> $MINTER"

echo "Generating .did for backend (later will also do nft and canisters(test canisters)"
generate-did InheritNext_backend

echo "Deploying Canisters"
dfx deploy InheritNext_backend

dfx deploy icrc1_ledger_canister --argument "(variant {
  Init = record {
    token_symbol = \"TST\";
    token_name = \"Test Token\";
    minting_account = record { owner = principal \"$MINTER\" };
    transfer_fee = 10_000;
    metadata = vec {};
    initial_balances = vec {};
    archive_options = record {
      num_blocks_to_archive = 1000;
      trigger_threshold = 2000;
      max_message_size_bytes = null;
      cycles_for_archive_creation = opt 1_000_000_000_000;
      node_max_memory_size_bytes = opt 3_221_225_472;
      controller_id = principal \"$MINTER\";
    };
    feature_flags = opt record {
      icrc2 = true;
    };
  }
})"

dfx deploy internet_identity
dfx deploy InheritNext_frontend

echo "Starting Frontend First Creating declarations"
npm run generate

npm run start
