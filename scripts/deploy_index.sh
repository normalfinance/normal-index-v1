# Ensure the script exits on any errors
set -e

# Usage
usage() {
    echo "Usage:"
    echo "  $0 <identity_string> <network> <public> <name> <symbol> <description> <initial_price> <initial_deposit> <manager_fee>"
    echo ""
    echo "Example:"
    echo "  $0 admin testnet true My_Index MIDX My_Description 100_0000000 100_0000000 0"
    exit 1
}

# Validate args
if [ "$#" -ne 9 ]; then
    usage
fi

# Parse arguments
IDENTITY_STRING="$1"
NETWORK=$2
PUBLIC="$3"
NAME="$4"
SYMBOL="$5"
DESCRIPTION="$6"
INITIAL_PRICE="$7"
INITIAL_DEPOSIT=$8
MANAGER_FEE=$9

# Load env vars dynamically
REPO_ROOT="$(git rev-parse --show-toplevel)"
source "$REPO_ROOT/scripts/load-env.sh" "$NETWORK"

cd target/wasm32v1-none/release

# Get admin address
ADMIN_ADDRESS=$(soroban keys address "$IDENTITY_STRING")

# Initialize index
echo "📦 Deploying an Index through Index Factory..."

# Setup the Stellar Classic asset

CLASSIC_ASSET="$SYMBOL:$ISSUER_ADDRESS"

# Issue an asset by creating a trustline
stellar tx new change-trust \
    --source-account "$DISTRIBUTOR" \
    --network "$NETWORK" \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    --line "$CLASSIC_ASSET"

# TODO: Serialize the asset
# serialized_assetis the Stellar Asset XDR serialized to bytes.
SERIALIZED_ASSET="$CLASSIC_ASSET"

# TODO: configure the index params
PARAMS=0

stellar contract invoke \
    --id "$INDEX_FACTORY_ADDR" \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK" \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- \
    deploy_index_contract \
    --serialized_asset $SERIALIZED_ASSET \
    --params "$PARAMS"
