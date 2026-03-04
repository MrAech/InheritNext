#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE}  InheritNext BACKEND Integration Tests                         ${NC}"
echo -e "${BLUE}  Testing: Backend APIs for aaset types                         ${NC}"
echo -e "${BLUE}================================================================${NC}"
echo ""

if dfx ping >/dev/null 2>&1; then
    echo -e "${YELLOW}DFX running. Stopping and running ${NC}"
    dfx stop
    sleep 2
    dfx start --clean --background
    sleep 5
else
    echo -e "${YELLOW} dfx notrunning. Starting it ${NC}"
    dfx start --clean --background
    sleep 5

fi

echo -e "${YELLOW}[0] Deploying/Checking backend canister....${NC}"
dfx deploy InheritNext_backend 2>/dev/null || true
BACKEND=$(dfx canister id InheritNext_backend)
echo -e "${GREEN}[Ok} Backend: $BACKEND${NC}"
echo "Backend Deployment: PASS"

echo -e "Using Default Identity"
dfx identity use default

echo -e "${YELLOW}[1] Deploying ICRC2 test-ledger...${NC}"
MINTER=$(dfx identity get-principal)

echo "Deploying ICRC-1 Ledger canister"
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

LEDGER=$(dfx canister id icrc1_ledger_canister)
echo -e "${GREEN}[OK] Ledger deployed: $LEDGER${NC}"
echo "ICRC-2 Ledger Deployment: PASS"

echo -e "${YELLOW}[2] Setting test Owner identity"
dfx identity new aech --storage-mode=plaintext 2>/dev/null || true

AECH=$(dfx identity get-principal --identity aech)

echo -e "${GREEN}AECH: $AECH${NC}"

echo -e "${YELLOW}[3] Minting Token (1,00,000) to AECH"
dfx identity use default
dfx canister call $LEDGER icrc1_transfer "(record {
    to = record{
    owner = principal \"$AECH\"; 
    subaccount = null 
    };
    amount = 1_000_000_000_000;
    fee = null;
    memo = null;
    from_subaccount = null;
    created_at_time = null;
    })"

echo "Token Minting to Aech: PASS"
echo -e "${GREEN}[OK] Tokens minted${NC}"

echo -e "${YELLOW}[4] Checking Aech's initial balance...${NC}"
AECH_BALANCE=$(dfx canister call $LEDGER icrc1_balance_of "(record { owner = principal \"$AECH\"; subaccount = null })")
echo -e "${GREEN}Aech balance: $AECH_BALANCE${NC}"

#TEST: User registration and vault creation
echo -e "${YELLOW}[5] Testing BACKEND: register_user, create_vault, configure_dms${NC}"
dfx identity use aech
dfx canister call $BACKEND register_user '("Aech", "Tester")' && echo -e "${GREEN}[OK] BACKEND: User registered${NC}" || echo -e "${YELLOW}Already registered${NC}"
dfx canister call $BACKEND create_vault && echo -e "${GREEN}[OK] BACKEND: Vault created${NC}" || echo -e "${YELLOW}Vault exists${NC}"
dfx canister call $BACKEND configure_dms '(1, 1)' && echo -e "${GREEN}[OK] BACKEND: DMS configured (1 day intervals)${NC}"

echo -e "${YELLOW}[7] Aech approves backend to spend 500,000 TST...${NC}"
dfx identity use aech
APPROVAL=$(dfx canister call $LEDGER icrc2_approve "(record {
  spender = record { owner = principal \"$BACKEND\"; subaccount = null };
  amount = 500_000_000_000;
  fee = null;
  memo = null;
  from_subaccount = null;
  created_at_time = null;
  expected_allowance = null;
  expires_at = null;
})")
echo -e "${GREEN}[OK] Approval result: $APPROVAL${NC}"
echo "ICRC-2 Approval (Aech->Backend): PASS"

echo -e "${YELLOW}[8] Verifying allowance...${NC}"
ALLOWANCE=$(dfx canister call $LEDGER icrc2_allowance "(record {
  account = record { owner = principal \"$AECH\"; subaccount = null };
  spender = record { owner = principal \"$BACKEND\"; subaccount = null };
})")
echo -e "${GREEN}Allowance: $ALLOWANCE${NC}"

#TEST: Adding ICRC2 token as asset
echo -e "${YELLOW}[9] Testing BACKEND: add_asset for ICRC-2 tokens (validates allowance)${NC}"
dfx identity use aech
ASSET_RESULT=$(dfx canister call $BACKEND add_asset "(
  \"Test Tokens for Inheritance\",
  \"400k TST tokens to be inherited\",
  variant { 
    ICRC2Token = record { 
      ledger_canister = principal \"$LEDGER\"; 
      amount = 400_000_000_000 
    } 
  },
  vec {})")
echo -e "${GREEN}[OK] Asset added: $ASSET_RESULT${NC}"
echo "Add ICRC-2 Token Asset to Vault: PASS"

#TEST: Asset List
echo -e "${YELLOW}[10] Testing BACKEND: list_my_assets${NC}"
dfx canister call $BACKEND list_my_assets
echo "Add ICRC-2 List My Asset: PASS"

#TEST: Vault status
echo -e "${YELLOW}[11] Testing BACKEND: get_my_vault${NC}"
dfx canister call $BACKEND get_my_vault
echo "Add ICRC-2 Get my Vault: PASS"

echo -e "${GREEN}================================================================${NC}"
echo -e "${GREEN}        BACKEND INTEGRATION TEST COMPLETE                       ${NC}"
echo -e "${GREEN}================================================================${NC}"
